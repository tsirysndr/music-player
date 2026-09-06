//! Jetstream subscription: several public instances at once, de-duplicated
//! into a single stream of commits.

use std::collections::{HashSet, VecDeque};
use std::time::Duration;

use anyhow::Error;
use futures_util::StreamExt;
use serde::Deserialize;

/// Public Jetstream instances followed in parallel. Any one of them can lag or
/// drop, so several are subscribed at once and the duplicate copies of each
/// event are filtered out on the way in (see [`Seen`]).
pub const JETSTREAM_ENDPOINTS: &[&str] = &[
    "wss://jetstream1.us-east.bsky.network/subscribe",
    "wss://jetstream2.us-east.bsky.network/subscribe",
    "wss://jetstream1.us-west.bsky.network/subscribe",
    "wss://jetstream2.us-west.bsky.network/subscribe",
];

/// How many recent event keys to remember when de-duplicating. Far more than
/// the fan-in of the streams can put in flight at once.
const SEEN_CAPACITY: usize = 1024;

/// Back-off between Jetstream reconnects. The firehose drops connections
/// routinely; reconnecting immediately in a tight loop would hammer it.
pub const RECONNECT_DELAY: Duration = Duration::from_secs(15);

#[derive(Debug, Deserialize)]
pub struct JetstreamEvent {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub commit: Option<JetstreamCommit>,
}

#[derive(Debug, Deserialize)]
pub struct JetstreamCommit {
    #[serde(default)]
    pub operation: String,
    #[serde(default)]
    pub collection: String,
    #[serde(default)]
    pub rkey: String,
    /// Repo revision of the commit. Unlike `time_us` — which each Jetstream
    /// instance stamps itself, so it differs between instances — the rev comes
    /// from the repo, so it is the same on every stream carrying this event.
    #[serde(default)]
    pub rev: String,
    #[serde(default)]
    pub cid: String,
    #[serde(default)]
    pub record: Option<serde_json::Value>,
}

impl JetstreamEvent {
    /// A key identifying this commit independently of which instance it came
    /// from, so the same event arriving on several streams is applied once.
    fn dedup_key(&self) -> Option<String> {
        let commit = self.commit.as_ref()?;
        // Pre-rev events (and any instance that omits it) still pin down a
        // single write with the record's cid.
        let identity = if commit.rev.is_empty() {
            &commit.cid
        } else {
            &commit.rev
        };
        Some(format!(
            "{identity}:{}:{}:{}",
            commit.collection, commit.rkey, commit.operation
        ))
    }
}

/// Bounded set of recently applied event keys, oldest evicted first.
#[derive(Default)]
struct Seen {
    keys: HashSet<String>,
    order: VecDeque<String>,
}

impl Seen {
    /// Record `key`, returning true when it had not been seen yet.
    fn insert(&mut self, key: String) -> bool {
        if !self.keys.insert(key.clone()) {
            return false;
        }
        self.order.push_back(key);
        if self.order.len() > SEEN_CAPACITY {
            if let Some(evicted) = self.order.pop_front() {
                self.keys.remove(&evicted);
            }
        }
        true
    }
}

/// The subscription query string.
///
/// `wantedCollections` is **repeated**, once per collection — Jetstream
/// validates each value as an NSID, so a comma-joined list is rejected outright
/// with `400 Bad Request` the moment there is more than one collection.
///
/// DIDs and NSIDs are drawn from `[a-z0-9:._-]`, all legal unescaped in a query
/// value, so nothing here needs percent-encoding.
fn subscribe_query(did: &str, collections: &[&str]) -> String {
    let mut query = format!("wantedDids={did}");
    for collection in collections {
        query.push_str("&wantedCollections=");
        query.push_str(collection);
    }
    query
}

/// Read one connection to `endpoint` until it closes, forwarding every event.
async fn stream_once(
    endpoint: &str,
    query: &str,
    events: &tokio::sync::mpsc::UnboundedSender<JetstreamEvent>,
) -> Result<(), Error> {
    let url = format!("{endpoint}?{query}");
    let (mut socket, _) = tokio_tungstenite::connect_async(&url).await?;
    tracing::info!(endpoint, query, "following the repo on Jetstream");
    while let Some(message) = socket.next().await {
        let text = match message? {
            tokio_tungstenite::tungstenite::Message::Text(text) => text,
            tokio_tungstenite::tungstenite::Message::Close(_) => break,
            _ => continue,
        };
        match serde_json::from_str::<JetstreamEvent>(&text) {
            Ok(event) => {
                if events.send(event).is_err() {
                    // The applier is gone; so is the reason to stay connected.
                    return Ok(());
                }
            }
            // Jetstream carries records this build doesn't model; skipping one
            // event must not tear the subscription down.
            Err(e) => tracing::debug!(endpoint, "skipping unparsable Jetstream event: {e}"),
        }
    }
    Ok(())
}

/// Keep one endpoint connected for as long as the daemon runs.
async fn follow_endpoint(
    endpoint: &'static str,
    query: String,
    events: tokio::sync::mpsc::UnboundedSender<JetstreamEvent>,
) {
    loop {
        if let Err(e) = stream_once(endpoint, &query, &events).await {
            tracing::warn!(endpoint, "Jetstream disconnected: {e}");
        }
        if events.is_closed() {
            return;
        }
        tokio::time::sleep(RECONNECT_DELAY).await;
    }
}

/// Subscribe to `collections` for `did` on every Jetstream endpoint at once and
/// call `apply` for each distinct commit, in arrival order. Runs until the
/// process exits.
pub async fn subscribe<F, Fut>(did: &str, collections: &[&str], mut apply: F)
where
    F: FnMut(JetstreamCommit) -> Fut,
    Fut: std::future::Future<Output = ()>,
{
    let query = subscribe_query(did, collections);
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    for endpoint in JETSTREAM_ENDPOINTS {
        tokio::spawn(follow_endpoint(endpoint, query.clone(), tx.clone()));
    }
    drop(tx);

    // Single applier: the streams overlap, so every event is filtered through
    // one dedup window before it reaches the caller.
    let mut seen = Seen::default();
    while let Some(event) = rx.recv().await {
        if event.kind != "commit" {
            continue;
        }
        let Some(key) = event.dedup_key() else {
            continue;
        };
        let Some(commit) = event.commit else {
            continue;
        };
        if !collections.contains(&commit.collection.as_str()) {
            continue;
        }
        if !seen.insert(key.clone()) {
            tracing::debug!(
                operation = %commit.operation,
                rkey = %commit.rkey,
                %key,
                "duplicate Jetstream event dropped"
            );
            continue;
        }
        tracing::info!(
            collection = %commit.collection,
            operation = %commit.operation,
            rkey = %commit.rkey,
            "new Jetstream event received"
        );
        apply(commit).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Jetstream validates every `wantedCollections` value as an NSID, so
    /// joining them with a comma made it reject the whole subscription with
    /// `400 Bad Request` as soon as there was more than one.
    #[test]
    fn repeats_wanted_collections_rather_than_joining_them() {
        assert_eq!(
            subscribe_query(
                "did:plc:abc",
                &["fm.atradio.favorite", "fm.atradio.station"]
            ),
            "wantedDids=did:plc:abc\
             &wantedCollections=fm.atradio.favorite\
             &wantedCollections=fm.atradio.station"
        );
        assert_eq!(
            subscribe_query("did:plc:abc", &["app.rocksky.like"]),
            "wantedDids=did:plc:abc&wantedCollections=app.rocksky.like"
        );
    }
}
