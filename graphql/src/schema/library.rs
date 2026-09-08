use async_graphql::*;
use music_player_storage::{
    repo::{album::AlbumRepository, artist::ArtistRepository, track::TrackRepository},
    searcher::Searcher,
    Database,
};
use std::sync::Arc;

use super::objects::{album::Album, artist::Artist, search_result::SearchResult, track::Track};
use super::provider::{self, decorate_all};

/// The library, from wherever it currently comes.
///
/// Every resolver has one shape: if a provider is connected, read from it and
/// absolutise its uris; otherwise read the local database. The check is a read
/// lock released before any request goes out, so a slow remote server cannot
/// stall a query that only touches local storage.
#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Track>, Error> {
        if let Some(current) = provider::connected(ctx).await {
            let tracks = current
                .provider
                .tracks(filter.as_deref(), provider::page(offset, limit))
                .await
                .map_err(provider::err)?;
            return Ok(decorate_all(tracks, &current.config)
                .into_iter()
                .map(Track::from)
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();
        let results = TrackRepository::new(db.get_connection())
            .find_all(
                filter,
                Some(offset.unwrap_or(0) as u64),
                limit.unwrap_or(100) as u64,
            )
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// The tracks the user has liked.
    ///
    /// On a remote provider these are *its* likes — Subsonic's starred songs,
    /// Jellyfin's favourites. It must not fall back to the local list: those
    /// ids belong to a different library, so the rows would render but not
    /// play. A provider with no notion of likes returns nothing, which is an
    /// empty screen rather than a wrong one.
    async fn liked_tracks(
        &self,
        ctx: &Context<'_>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Track>, Error> {
        if let Some(current) = provider::connected(ctx).await {
            let tracks = current
                .provider
                .liked_tracks(provider::page(offset, limit))
                .await
                .map_err(provider::err)?;
            return Ok(decorate_all(tracks, &current.config)
                .into_iter()
                .map(Track::from)
                .collect());
        }

        // Locally a like lives in the user's atproto repo (see
        // `music_player_storage::rocksky_likes`); only the ones matched to a
        // local file have a track to return, so an unmatched like is absent.
        let db = ctx.data::<Database>().unwrap();
        let ids = music_player_storage::rocksky_likes::matched_track_ids(db.get_connection())
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        let offset = offset.unwrap_or(0).max(0) as usize;
        let limit = limit.unwrap_or(100).max(0) as usize;
        let repository = TrackRepository::new(db.get_connection());
        let mut tracks = Vec::new();
        for id in ids.into_iter().skip(offset).take(limit) {
            // A like can outlive the file it matched (a rescan may drop it);
            // skip those rather than failing the whole query.
            if let Ok(track) = repository.find(&id).await {
                tracks.push(track.into());
            }
        }
        Ok(tracks)
    }

    async fn artists(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Artist>, Error> {
        if let Some(current) = provider::connected(ctx).await {
            let artists = current
                .provider
                .artists(filter.as_deref(), provider::page(offset, limit))
                .await
                .map_err(provider::err)?;
            return Ok(decorate_all(artists, &current.config)
                .into_iter()
                .map(Artist::from)
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();
        let results = ArtistRepository::new(db.get_connection())
            .find_all(filter, offset.map(|x| x as u64), limit.map(|x| x as u64))
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn albums(
        &self,
        ctx: &Context<'_>,
        filter: Option<String>,
        offset: Option<i32>,
        limit: Option<i32>,
    ) -> Result<Vec<Album>, Error> {
        if let Some(current) = provider::connected(ctx).await {
            let albums = current
                .provider
                .albums(filter.as_deref(), provider::page(offset, limit))
                .await
                .map_err(provider::err)?;
            return Ok(decorate_all(albums, &current.config)
                .into_iter()
                .map(Album::from)
                .collect());
        }

        let db = ctx.data::<Database>().unwrap();
        let results = AlbumRepository::new(db.get_connection())
            .find_all(filter, offset.map(|x| x as u64), limit.map(|x| x as u64))
            .await?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    async fn track(&self, ctx: &Context<'_>, id: ID) -> Result<Track, Error> {
        let id = id.to_string();
        if let Some(current) = provider::connected(ctx).await {
            let track = current.provider.track(&id).await.map_err(provider::err)?;
            return Ok(Track::from(provider::decorate(track, &current.config)));
        }

        let db = ctx.data::<Database>().unwrap();
        let track = TrackRepository::new(db.get_connection()).find(&id).await?;

        Ok(track.into())
    }

    async fn artist(&self, ctx: &Context<'_>, id: ID) -> Result<Artist, Error> {
        let id = id.to_string();
        if let Some(current) = provider::connected(ctx).await {
            let artist = current.provider.artist(&id).await.map_err(provider::err)?;
            return Ok(Artist::from(provider::decorate(artist, &current.config)));
        }

        let db = ctx.data::<Database>().unwrap();
        let artist = ArtistRepository::new(db.get_connection()).find(&id).await?;

        Ok(artist.into())
    }

    async fn album(&self, ctx: &Context<'_>, id: ID) -> Result<Album, Error> {
        let id = id.to_string();
        if let Some(current) = provider::connected(ctx).await {
            let album = current.provider.album(&id).await.map_err(provider::err)?;
            return Ok(Album::from(provider::decorate(album, &current.config)));
        }

        let db = ctx.data::<Database>().unwrap();
        let album = AlbumRepository::new(db.get_connection()).find(&id).await?;

        Ok(album.into())
    }

    /// Search wherever the library currently is.
    ///
    /// The local searcher indexes local files, so with a provider connected it
    /// would be answering about a library the user is not looking at.
    /// Search both libraries at once.
    ///
    /// With a provider connected the results are *federated*: the remote
    /// server and this machine's own index are queried together and the rows
    /// interleaved, so one search box covers everything reachable rather than
    /// silently describing only whichever library happens to be current.
    ///
    /// The two run concurrently — a remote round trip should not be paid on
    /// top of a local index scan — and a failure on either side yields that
    /// side's results as empty rather than failing the whole search: half an
    /// answer is worth more than none.
    async fn search(&self, ctx: &Context<'_>, keyword: String) -> Result<SearchResult, Error> {
        let searcher = ctx.data::<Arc<Searcher>>().unwrap();
        let local = async {
            SearchResult {
                artists: searcher
                    .search_artist(&keyword)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                albums: searcher
                    .search_album(&keyword)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                tracks: searcher
                    .search_song(&keyword)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }
        };

        let Some(current) = provider::connected(ctx).await else {
            return Ok(local.await);
        };

        let remote = async {
            match current
                .provider
                .search(&keyword, provider::page(None, Some(50)))
                .await
            {
                Ok(results) => SearchResult {
                    artists: decorate_all(results.artists, &current.config)
                        .into_iter()
                        .map(Artist::from)
                        .collect(),
                    albums: decorate_all(results.albums, &current.config)
                        .into_iter()
                        .map(Album::from)
                        .collect(),
                    tracks: decorate_all(results.tracks, &current.config)
                        .into_iter()
                        .map(Track::from)
                        .collect(),
                },
                Err(e) => {
                    tracing::warn!("searching {} failed: {e}", current.config.name);
                    SearchResult::default()
                }
            }
        };

        let (remote, local) = futures_util::future::join(remote, local).await;
        // The connected server leads: it is the library the screens are
        // showing, so it is what the user is most likely looking for.
        Ok(SearchResult {
            artists: [remote.artists, local.artists].concat(),
            albums: [remote.albums, local.albums].concat(),
            tracks: [remote.tracks, local.tracks].concat(),
        })
    }
}

#[derive(Default)]
pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    /// Like or unlike a track.
    ///
    /// With a provider connected the like belongs to *that* server, so it goes
    /// there. A provider with no notion of likes says so rather than quietly
    /// writing an id to Rocksky that means nothing there.
    async fn like_track(&self, ctx: &Context<'_>, id: String, like: bool) -> Result<bool, Error> {
        if let Some(current) = provider::connected(ctx).await {
            current
                .provider
                .set_liked(&id, like)
                .await
                .map_err(provider::err)?;
            return Ok(like);
        }

        // Locally the like lives with the client; a missing `rocksky login`
        // token makes this a silent no-op.
        let db = ctx.data::<Database>().unwrap();
        music_player_storage::rocksky::sync_like(db, id, like);
        Ok(like)
    }

    /// Always the local music directory: a scan is about the files on this
    /// machine, whatever the screens happen to be pointed at.
    async fn scan(&self, _ctx: &Context<'_>) -> Result<bool, Error> {
        music_player_scanner::refresh_music_library(false, Database::new().await)
            .await
            .map_err(|e| Error::new(e.to_string()))?;

        Ok(false)
    }
}
