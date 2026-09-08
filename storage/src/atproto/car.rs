//! CARv1 repo archives: download, block indexing, and MST traversal.
//!
//! `com.atproto.sync.getRepo` returns the whole repo in one request, which
//! beats paging `listRecords` per collection. Walking the Merkle Search Tree
//! (rather than block-scanning) is what gives each record the `collection/rkey`
//! path it is stored under, and therefore its `at://` uri.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Error};
use ipld_core::ipld::Ipld;
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use super::{http, repo::pds_endpoint};

/// How long a downloaded archive is reused. Long enough for the startup
/// imports — radio bookmarks and liked songs both read the same repo — and
/// short enough that a later read still sees a current repo.
const CAR_TTL: Duration = Duration::from_secs(300);

/// The last archive downloaded, shared by every caller within [`CAR_TTL`].
/// The lock is held across the download, so concurrent callers wait for the
/// one request in flight instead of starting their own.
static CACHE: Mutex<Option<CachedCar>> = Mutex::const_new(None);

struct CachedCar {
    did: String,
    fetched_at: Instant,
    car: Arc<Vec<u8>>,
}

/// `did`'s whole repo as a CARv1 archive — one request instead of paging
/// `listRecords` per collection, and one download shared between the
/// collections that are read out of it.
pub async fn get_repo_car(did: &str) -> Result<Arc<Vec<u8>>, Error> {
    let mut cache = CACHE.lock().await;
    if let Some(cached) = cache.as_ref() {
        if cached.did == did && cached.fetched_at.elapsed() < CAR_TTL {
            tracing::debug!(bytes = cached.car.len(), %did, "reusing the repo CAR archive");
            return Ok(cached.car.clone());
        }
    }
    // Stale or for another account: drop it before downloading, so a failed
    // download does not leave a whole repo in memory.
    *cache = None;

    let car = Arc::new(download_repo_car(did).await?);
    *cache = Some(CachedCar {
        did: did.to_owned(),
        fetched_at: Instant::now(),
        car: car.clone(),
    });
    // Nothing reads the repo again after the startup imports, so a timer
    // releases the archive rather than leaving it resident for the process.
    tokio::spawn(expire_after(CAR_TTL));
    Ok(car)
}

/// The archive already in memory for `did`, if one was downloaded recently.
/// Lets a caller reuse this run's download without consulting any other
/// policy — the collections read out of a repo are read in the same run.
pub async fn cached_repo_car(did: &str) -> Option<Arc<Vec<u8>>> {
    let cache = CACHE.lock().await;
    cache
        .as_ref()
        .filter(|cached| cached.did == did && cached.fetched_at.elapsed() < CAR_TTL)
        .map(|cached| cached.car.clone())
}

async fn download_repo_car(did: &str) -> Result<Vec<u8>, Error> {
    let pds = pds_endpoint(did).await?;
    tracing::info!(%did, %pds, "downloading the repo CAR archive");
    let car = http()?
        .get(format!("{pds}/xrpc/com.atproto.sync.getRepo"))
        .query(&[("did", did)])
        .send()
        .await?
        .error_for_status()?
        .bytes()
        .await?;
    tracing::info!(bytes = car.len(), %did, "repo CAR archive downloaded");
    Ok(car.to_vec())
}

/// Drop the cached archive once it has aged out.
async fn expire_after(ttl: Duration) {
    tokio::time::sleep(ttl).await;
    let mut cache = CACHE.lock().await;
    if cache
        .as_ref()
        .is_some_and(|cached| cached.fetched_at.elapsed() >= ttl)
    {
        *cache = None;
    }
}

/// Read an unsigned LEB128 varint, returning the value and its width.
fn read_varint(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut value: usize = 0;
    for (i, byte) in bytes.iter().take(9).enumerate() {
        value |= ((byte & 0x7f) as usize) << (i * 7);
        if byte & 0x80 == 0 {
            return Some((value, i + 1));
        }
    }
    None
}

/// A CAR archive's roots, and its blocks keyed by CID.
type CarBlocks = (Vec<cid::Cid>, HashMap<cid::Cid, Vec<u8>>);

/// Index every block in a CARv1 archive by CID.
fn car_blocks(car: &[u8]) -> Result<CarBlocks, Error> {
    let (header_len, width) = read_varint(car).ok_or_else(|| anyhow!("truncated CAR header"))?;
    let header_end = width
        .checked_add(header_len)
        .filter(|end| *end <= car.len())
        .ok_or_else(|| anyhow!("truncated CAR header"))?;

    // The header names the repo's root, i.e. the signed commit block.
    let header: Ipld = serde_ipld_dagcbor::from_slice(&car[width..header_end])
        .map_err(|e| anyhow!("unreadable CAR header: {e}"))?;
    let roots = match header {
        Ipld::Map(map) => match map.get("roots") {
            Some(Ipld::List(roots)) => roots
                .iter()
                .filter_map(|root| match root {
                    Ipld::Link(cid) => Some(*cid),
                    _ => None,
                })
                .collect::<Vec<_>>(),
            _ => Vec::new(),
        },
        _ => Vec::new(),
    };

    let mut offset = header_end;
    let mut blocks = HashMap::new();
    while offset < car.len() {
        let Some((block_len, width)) = read_varint(&car[offset..]) else {
            break;
        };
        offset += width;
        let Some(block) = car.get(offset..offset + block_len) else {
            break;
        };
        offset += block_len;

        // Each block is a CID followed by its payload.
        let mut reader = block;
        let Ok(cid) = cid::Cid::read_bytes(&mut reader) else {
            continue;
        };
        blocks.insert(cid, block[block.len() - reader.len()..].to_vec());
    }
    Ok((roots, blocks))
}

/// Walk the repo's Merkle Search Tree, collecting every `collection/rkey` path
/// and the CID of the record it points at.
///
/// MST nodes hold entries with prefix-compressed keys: `p` bytes shared with
/// the previous key, then the `k` suffix. `l` is the subtree left of all
/// entries, `t` the subtree right of one entry.
fn walk_mst(
    blocks: &HashMap<cid::Cid, Vec<u8>>,
    node: &cid::Cid,
    depth: usize,
    out: &mut Vec<(String, cid::Cid)>,
) {
    // Repos are shallow (a few levels); this only guards against a cycle in a
    // malformed archive.
    if depth > 64 {
        return;
    }
    let Some(payload) = blocks.get(node) else {
        return;
    };
    let Ok(Ipld::Map(map)) = serde_ipld_dagcbor::from_slice::<Ipld>(payload) else {
        return;
    };
    if let Some(Ipld::Link(left)) = map.get("l") {
        walk_mst(blocks, left, depth + 1, out);
    }
    let Some(Ipld::List(entries)) = map.get("e") else {
        return;
    };
    let mut previous: Vec<u8> = Vec::new();
    for entry in entries {
        let Ipld::Map(entry) = entry else { continue };
        let (Some(Ipld::Integer(prefix)), Some(Ipld::Bytes(suffix)), Some(Ipld::Link(value))) =
            (entry.get("p"), entry.get("k"), entry.get("v"))
        else {
            continue;
        };
        let prefix = (*prefix).max(0) as usize;
        if prefix > previous.len() {
            continue;
        }
        let mut key = previous[..prefix].to_vec();
        key.extend_from_slice(suffix);
        previous = key.clone();
        if let Ok(path) = String::from_utf8(key) {
            out.push((path, *value));
        }
        if let Some(Ipld::Link(right)) = entry.get("t") {
            walk_mst(blocks, right, depth + 1, out);
        }
    }
}

/// Every record of `collection` in a repo CAR archive, as `(rkey, payload)`.
///
/// The archive is walked through its MST rather than block-scanned, so each
/// record comes back with the path it is stored under — which is what makes
/// its `at://` uri reconstructable.
pub fn car_records(car: &[u8], collection: &str) -> Result<Vec<(String, Vec<u8>)>, Error> {
    let (roots, blocks) = car_blocks(car)?;
    let root = roots
        .first()
        .ok_or_else(|| anyhow!("CAR archive names no root commit"))?;
    // The signed commit points at the MST root through `data`.
    let commit: Ipld = serde_ipld_dagcbor::from_slice(
        blocks
            .get(root)
            .ok_or_else(|| anyhow!("the CAR root commit block is missing"))?,
    )
    .map_err(|e| anyhow!("unreadable commit block: {e}"))?;
    let Ipld::Map(commit) = commit else {
        return Err(anyhow!("the CAR root is not a commit"));
    };
    let Some(Ipld::Link(data)) = commit.get("data") else {
        return Err(anyhow!("the commit has no MST root"));
    };

    let mut paths = Vec::new();
    walk_mst(&blocks, data, 0, &mut paths);

    let prefix = format!("{collection}/");
    let out: Vec<(String, Vec<u8>)> = paths
        .into_iter()
        .filter_map(|(path, cid)| {
            let rkey = path.strip_prefix(&prefix)?;
            Some((rkey.to_owned(), blocks.get(&cid)?.clone()))
        })
        .collect();
    tracing::info!(
        blocks = blocks.len(),
        found = out.len(),
        collection,
        "scanned the repo CAR archive"
    );
    Ok(out)
}

/// Every record of `collection` in the archive, decoded as `T`, keyed by the
/// `at://` uri it lives at in `did`'s repo.
pub fn records_from_car<T: DeserializeOwned>(
    car: &[u8],
    did: &str,
    collection: &str,
) -> Result<Vec<(String, T)>, Error> {
    Ok(car_records(car, collection)?
        .into_iter()
        .filter_map(|(rkey, payload)| {
            let record = serde_ipld_dagcbor::from_slice::<T>(&payload).ok()?;
            Some((format!("at://{did}/{collection}/{rkey}"), record))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_varints() {
        assert_eq!(read_varint(&[0x00]), Some((0, 1)));
        assert_eq!(read_varint(&[0x7f]), Some((127, 1)));
        assert_eq!(read_varint(&[0x80, 0x01]), Some((128, 2)));
        assert_eq!(read_varint(&[0xac, 0x02]), Some((300, 2)));
    }
}
