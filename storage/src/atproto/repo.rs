//! Public repo reads: PDS discovery, `listRecords`, `getRecord`, and at-uri
//! handling. None of these need a session.

use anyhow::{anyhow, Error};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use super::http;

/// Resolve the DID document and pull out the account's PDS endpoint.
pub async fn pds_endpoint(did: &str) -> Result<String, Error> {
    let url = if let Some(host) = did.strip_prefix("did:web:") {
        format!("https://{host}/.well-known/did.json")
    } else {
        format!("https://plc.directory/{did}")
    };
    let doc: serde_json::Value = http()?
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    doc.get("service")
        .and_then(|services| services.as_array())
        .and_then(|services| {
            services.iter().find(|service| {
                service
                    .get("id")
                    .and_then(|id| id.as_str())
                    .is_some_and(|id| id.ends_with("#atproto_pds"))
            })
        })
        .and_then(|service| service.get("serviceEndpoint"))
        .and_then(|endpoint| endpoint.as_str())
        .map(|endpoint| endpoint.trim_end_matches('/').to_owned())
        .ok_or_else(|| anyhow!("no atproto PDS in the DID document of {did}"))
}

/// Every record of `collection` in `did`'s repo, paginated, as
/// `(record uri, value)`. Public read: no session needed.
pub async fn list_records<T: DeserializeOwned>(
    did: &str,
    collection: &str,
) -> Result<Vec<(String, T)>, Error> {
    #[derive(Deserialize)]
    struct Page<T> {
        records: Vec<Entry<T>>,
        #[serde(default)]
        cursor: Option<String>,
    }
    #[derive(Deserialize)]
    struct Entry<T> {
        uri: String,
        value: T,
    }

    let pds = pds_endpoint(did).await?;
    let client = http()?;
    let mut out = Vec::new();
    let mut cursor: Option<String> = None;
    loop {
        let mut request = client
            .get(format!("{pds}/xrpc/com.atproto.repo.listRecords"))
            .query(&[("repo", did), ("collection", collection), ("limit", "100")]);
        if let Some(cursor) = &cursor {
            request = request.query(&[("cursor", cursor)]);
        }
        let page: Page<T> = request.send().await?.error_for_status()?.json().await?;
        let empty = page.records.is_empty();
        out.extend(
            page.records
                .into_iter()
                .map(|entry| (entry.uri, entry.value)),
        );
        match page.cursor {
            Some(next) if !empty => cursor = Some(next),
            _ => break,
        }
    }
    Ok(out)
}

/// Fetch one record by its `at://did/collection/rkey` uri.
pub async fn get_record<T: DeserializeOwned>(uri: &str) -> Result<T, Error> {
    #[derive(Deserialize)]
    struct Output<T> {
        value: T,
    }
    let (did, collection, rkey) =
        split_at_uri(uri).ok_or_else(|| anyhow!("not an at:// record uri: {uri}"))?;
    let pds = pds_endpoint(did).await?;
    let output: Output<T> = http()?
        .get(format!("{pds}/xrpc/com.atproto.repo.getRecord"))
        .query(&[("repo", did), ("collection", collection), ("rkey", rkey)])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    Ok(output.value)
}

/// Split `at://did/collection/rkey` into its three parts.
pub fn split_at_uri(uri: &str) -> Option<(&str, &str, &str)> {
    let rest = uri.strip_prefix("at://")?;
    let mut parts = rest.splitn(3, '/');
    let did = parts.next()?;
    let collection = parts.next()?;
    let rkey = parts.next()?;
    (!did.is_empty() && !collection.is_empty() && !rkey.is_empty())
        .then_some((did, collection, rkey))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_at_uris() {
        assert_eq!(
            split_at_uri("at://did:plc:abc/app.rocksky.song/3mub"),
            Some(("did:plc:abc", "app.rocksky.song", "3mub"))
        );
        assert_eq!(split_at_uri("https://example.com"), None);
        assert_eq!(split_at_uri("at://did:plc:abc/app.rocksky.song"), None);
    }
}
