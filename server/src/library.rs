use music_player_provider::{Page, ProviderError, ProviderState};
use music_player_storage::repo::album::AlbumRepository;
use music_player_storage::repo::artist::ArtistRepository;
use music_player_storage::repo::track::TrackRepository;
use music_player_storage::searcher::Searcher;
use music_player_storage::Database;
use std::sync::Arc;

use crate::api::metadata::v1alpha1::{
    Album as AlbumMetadata, Artist as ArtistMetadata, Track as TrackMetadata,
};
use crate::api::music::v1alpha1::{
    library_service_server::LibraryService, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetTrackDetailsRequest, GetTrackDetailsResponse,
    GetLikedTracksRequest, GetLikedTracksResponse, GetTracksRequest, GetTracksResponse,
    LikeTrackRequest, LikeTrackResponse, ScanRequest,
    ScanResponse, SearchRequest, SearchResponse,
};

pub struct Library {
    db: Database,
    /// Where the library is read from. `None` current means local files.
    ///
    /// Shared with the GraphQL schema, so the two API surfaces cannot
    /// disagree about which server a client is looking at — which is exactly
    /// what happened while only GraphQL knew: the Slint desktop and the TUI
    /// read over gRPC and kept showing the local library after a switch.
    providers: Arc<ProviderState>,
}

impl Library {
    pub fn new(db: Database, providers: Arc<ProviderState>) -> Self {
        Self { db, providers }
    }
}

/// A provider failure as a gRPC status, keeping the distinctions the trait
/// draws: a wrong password is not the same as a server that is down.
pub(crate) fn provider_status(e: ProviderError) -> tonic::Status {
    match e {
        ProviderError::NotFound(what) => tonic::Status::not_found(what),
        ProviderError::Auth(why) => tonic::Status::unauthenticated(why),
        ProviderError::Unsupported { .. } => tonic::Status::failed_precondition(e.to_string()),
        other => tonic::Status::unavailable(other.to_string()),
    }
}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn scan(
        &self,
        _request: tonic::Request<ScanRequest>,
    ) -> Result<tonic::Response<ScanResponse>, tonic::Status> {
        music_player_scanner::refresh_music_library(false, self.db.clone())
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let response = ScanResponse {};
        Ok(tonic::Response::new(response))
    }

    async fn search(
        &self,
        request: tonic::Request<SearchRequest>,
    ) -> Result<tonic::Response<SearchResponse>, tonic::Status> {
        let query = request.into_inner().query;

        // The local index describes local files, so with a provider connected
        // it would be answering about a library nobody is looking at.
        if let Some(current) = self.providers.current().await {
            let results = current
                .provider
                .search(&query, Page::new(0, 50))
                .await
                .map_err(provider_status)?;
            let config = &current.config;
            return Ok(tonic::Response::new(SearchResponse {
                tracks: music_player_provider::url::decorate_all(results.tracks, config)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                albums: music_player_provider::url::decorate_all(results.albums, config)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
                artists: music_player_provider::url::decorate_all(results.artists, config)
                    .into_iter()
                    .map(Into::into)
                    .collect(),
            }));
        }

        let searcher = Searcher::new(self.db.get_connection().clone());

        let tracks = searcher
            .search_song(&query)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let albums = searcher
            .search_album(&query)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;
        let artists = searcher
            .search_artist(&query)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = SearchResponse {
            tracks: tracks
                .into_iter()
                .map(|song| TrackMetadata {
                    id: song.id,
                    title: song.title,
                    artist: song.artist,
                    duration: song.duration.as_secs_f32(),
                    album: Some(AlbumMetadata {
                        id: song.album_id,
                        title: song.album,
                        cover: song.cover.unwrap_or_default(),
                        ..Default::default()
                    }),
                    ..Default::default()
                })
                .collect(),
            albums: albums
                .into_iter()
                .map(|album| AlbumMetadata {
                    id: album.id,
                    title: album.title,
                    artist: album.artist,
                    year: album.year.map(|year| year as i32).unwrap_or_default(),
                    cover: album.cover.unwrap_or_default(),
                    ..Default::default()
                })
                .collect(),
            artists: artists
                .into_iter()
                .map(|artist| ArtistMetadata {
                    id: artist.id,
                    name: artist.name,
                    ..Default::default()
                })
                .collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_artists(
        &self,
        request: tonic::Request<GetArtistsRequest>,
    ) -> Result<tonic::Response<GetArtistsResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = request.limit;

        if let Some(current) = self.providers.current().await {
            let artists = current
                .provider
                .artists(filter.as_deref(), Page::new(offset, limit))
                .await
                .map_err(provider_status)?;
            let artists = music_player_provider::url::decorate_all(artists, &current.config);
            return Ok(tonic::Response::new(GetArtistsResponse {
                artists: artists.into_iter().map(Into::into).collect(),
            }));
        }

        let results = ArtistRepository::new(self.db.get_connection())
            .find_all(filter, Some(offset as u64), Some(limit as u64))
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistsResponse {
            artists: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_albums(
        &self,
        request: tonic::Request<GetAlbumsRequest>,
    ) -> Result<tonic::Response<GetAlbumsResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = request.limit;

        if let Some(current) = self.providers.current().await {
            let albums = current
                .provider
                .albums(filter.as_deref(), Page::new(offset, limit))
                .await
                .map_err(provider_status)?;
            let albums = music_player_provider::url::decorate_all(albums, &current.config);
            return Ok(tonic::Response::new(GetAlbumsResponse {
                albums: albums.into_iter().map(Into::into).collect(),
            }));
        }

        let results = AlbumRepository::new(self.db.get_connection())
            .find_all(filter, Some(offset as u64), Some(limit as u64))
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetAlbumsResponse {
            albums: results.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    async fn get_tracks(
        &self,
        request: tonic::Request<GetTracksRequest>,
    ) -> Result<tonic::Response<GetTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let filter = match request.filter.as_str() {
            "" => None,
            _ => Some(request.filter),
        };
        let offset = request.offset;
        let limit = match request.limit {
            0 => 100,
            _ => request.limit,
        };
        if let Some(current) = self.providers.current().await {
            let tracks = current
                .provider
                .tracks(filter.as_deref(), Page::new(offset, limit))
                .await
                .map_err(provider_status)?;
            let tracks = music_player_provider::url::decorate_all(tracks, &current.config);
            return Ok(tonic::Response::new(GetTracksResponse {
                tracks: tracks.into_iter().map(Into::into).collect(),
            }));
        }

        let tracks = TrackRepository::new(self.db.get_connection())
            .find_all(filter, Some(offset as u64), limit as u64)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetTracksResponse {
            tracks: tracks.into_iter().map(Into::into).collect(),
        };
        Ok(tonic::Response::new(response))
    }

    /// The tracks the user has liked.
    ///
    /// On a remote provider these are *its* likes — Subsonic's starred songs,
    /// Jellyfin's favourites. There is deliberately no fallback to the local
    /// list: those ids belong to a different library, so the rows would render
    /// but not play.
    async fn get_liked_tracks(
        &self,
        request: tonic::Request<GetLikedTracksRequest>,
    ) -> Result<tonic::Response<GetLikedTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        let page = Page::new(request.offset as i32, request.limit as i32);

        if let Some(current) = self.providers.current().await {
            let tracks = current
                .provider
                .liked_tracks(page)
                .await
                .map_err(provider_status)?;
            let tracks = music_player_provider::url::decorate_all(tracks, &current.config);
            return Ok(tonic::Response::new(GetLikedTracksResponse {
                tracks: tracks.into_iter().map(Into::into).collect(),
            }));
        }

        // Locally a like lives in the user's atproto repo; only the ones
        // matched to a local file have a track to return.
        let ids = music_player_storage::rocksky_likes::matched_track_ids(
            self.db.get_connection(),
        )
        .await
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let repository = TrackRepository::new(self.db.get_connection());
        let mut tracks = Vec::new();
        for id in ids
            .into_iter()
            .skip(page.offset.max(0) as usize)
            .take(if page.limit > 0 {
                page.limit as usize
            } else {
                usize::MAX
            })
        {
            // A like can outlive the file it matched; skip those rather than
            // failing the whole call.
            if let Ok(track) = repository.find(&id).await {
                tracks.push(track.into());
            }
        }
        Ok(tonic::Response::new(GetLikedTracksResponse { tracks }))
    }

    async fn like_track(
        &self,
        request: tonic::Request<LikeTrackRequest>,
    ) -> Result<tonic::Response<LikeTrackResponse>, tonic::Status> {
        let request = request.into_inner();
        // With a provider connected the like belongs to *that* server: a
        // remote id means nothing to Rocksky.
        if let Some(current) = self.providers.current().await {
            current
                .provider
                .set_liked(&request.id, request.like)
                .await
                .map_err(provider_status)?;
            return Ok(tonic::Response::new(LikeTrackResponse {}));
        }
        music_player_storage::rocksky::sync_like(&self.db, request.id, request.like);
        Ok(tonic::Response::new(LikeTrackResponse {}))
    }

    async fn get_track_details(
        &self,
        request: tonic::Request<GetTrackDetailsRequest>,
    ) -> Result<tonic::Response<GetTrackDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;

        if let Some(current) = self.providers.current().await {
            let track = current.provider.track(&id).await.map_err(provider_status)?;
            let track = music_player_provider::url::decorate(track, &current.config);
            return Ok(tonic::Response::new(GetTrackDetailsResponse {
                track: Some(track.into()),
            }));
        }

        let track = TrackRepository::new(self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GetTrackDetailsResponse {
            track: Some(track.into()),
        }))
    }

    async fn get_album_details(
        &self,
        request: tonic::Request<GetAlbumDetailsRequest>,
    ) -> Result<tonic::Response<GetAlbumDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;

        if let Some(current) = self.providers.current().await {
            let album = current.provider.album(&id).await.map_err(provider_status)?;
            let album = music_player_provider::url::decorate(album, &current.config);
            return Ok(tonic::Response::new(GetAlbumDetailsResponse {
                album: Some(album.into()),
            }));
        }

        let album = AlbumRepository::new(self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(GetAlbumDetailsResponse {
            album: Some(album.into()),
        }))
    }

    async fn get_artist_details(
        &self,
        request: tonic::Request<GetArtistDetailsRequest>,
    ) -> Result<tonic::Response<GetArtistDetailsResponse>, tonic::Status> {
        let id = request.into_inner().id;

        if let Some(current) = self.providers.current().await {
            let artist = current
                .provider
                .artist(&id)
                .await
                .map_err(provider_status)?;
            let artist = music_player_provider::url::decorate(artist, &current.config);
            return Ok(tonic::Response::new(GetArtistDetailsResponse {
                artist: Some(artist.into()),
            }));
        }

        let artist = ArtistRepository::new(self.db.get_connection())
            .find(&id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        let response = GetArtistDetailsResponse {
            artist: Some(artist.into()),
        };
        Ok(tonic::Response::new(response))
    }
}
