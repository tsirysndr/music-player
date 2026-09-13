//! Reaching the connected provider from a resolver.
//!
//! Every library resolver has the same shape — read from the connected server
//! if there is one, otherwise from the local database — and these keep that
//! shape to two lines instead of fifteen.
//!
//! Note what [`connected`] does *not* do: it takes a read lock, clones two
//! `Arc`s and lets go. Requests then run with no lock held, so a slow remote
//! server can no longer stall every other resolver — including ones that only
//! ever touch the local library, which is what the single global mutex this
//! replaces used to do.

use async_graphql::{Context, Error};
use music_player_provider::{
    ConnectedProvider, Page, ProviderConfig, ProviderError, ProviderState,
};
use music_player_types::types::{RemoteCoverUrl, RemoteTrackUrl};
use std::sync::Arc;

pub fn state<'a>(ctx: &'a Context<'_>) -> &'a Arc<ProviderState> {
    ctx.data::<Arc<ProviderState>>()
        .expect("the provider state is registered on the schema")
}

/// The connected provider, or `None` for the local library.
pub async fn connected(ctx: &Context<'_>) -> Option<ConnectedProvider> {
    state(ctx).current().await
}

/// A `ProviderError` as something GraphQL can return.
pub fn err(e: ProviderError) -> Error {
    Error::new(e.to_string())
}

/// Refresh the queued tracks' `liked` from a freshly connected provider's
/// starred list — the queued copies are snapshots from when each track was
/// queued (or from a restored session), so a star set since then never
/// reaches the now-playing heart without this.
///
/// Detached: the connect reply must not wait on a full starred-list fetch,
/// and a failure only means the hearts keep their queue-time snapshots.
pub fn restamp_queue_likes(ctx: &Context<'_>, connected: ConnectedProvider) {
    let tracklist = ctx
        .data::<Arc<std::sync::Mutex<music_player_tracklist::Tracklist>>>()
        .expect("the tracklist is registered on the schema")
        .clone();
    tokio::spawn(async move {
        match connected.provider.liked_tracks(Page::new(0, 0)).await {
            Ok(starred) => {
                let ids = starred.into_iter().map(|track| track.id).collect();
                tracklist.lock().unwrap().restamp_liked(&ids);
            }
            Err(e) => tracing::warn!("could not refresh queued likes: {e}"),
        }
    });
}

/// The paging arguments every listing resolver takes.
pub fn page(offset: Option<i32>, limit: Option<i32>) -> Page {
    Page::new(offset.unwrap_or(0), limit.unwrap_or(100))
}

/// Absolutise a remote item's track and cover uris against its provider.
///
/// One place, so the GraphQL and gRPC paths cannot decorate differently. Uris
/// that are already absolute — a signed Subsonic stream link — are left alone
/// by the underlying impls; rewriting one would strip its token.
pub fn decorate<T>(item: T, config: &ProviderConfig) -> T
where
    T: RemoteTrackUrl + RemoteCoverUrl,
{
    music_player_provider::url::decorate(item, config)
}

pub fn decorate_all<T>(items: Vec<T>, config: &ProviderConfig) -> Vec<T>
where
    T: RemoteTrackUrl + RemoteCoverUrl,
{
    music_player_provider::url::decorate_all(items, config)
}
