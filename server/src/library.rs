use music_player_playback::source_resolver::SourceResolver;
use music_player_provider::{ConnectedProvider, Page, ProviderError, ProviderState};
use music_player_storage::repo::album::AlbumRepository;
use music_player_storage::repo::artist::ArtistRepository;
use music_player_storage::repo::track::TrackRepository;
use music_player_storage::searcher::Searcher;
use music_player_storage::Database;
use music_player_tracklist::Tracklist as TracklistState;
use music_player_types::{audio, source::SourceHandle};
use std::sync::Arc;

use crate::api::metadata::v1alpha1::{
    Album as AlbumMetadata, Artist as ArtistMetadata, Track as TrackMetadata,
};
use crate::api::music::v1alpha1::{
    library_service_server::LibraryService, AudioStreamOption, AudioVersionOption,
    BrowseMediaRequest, BrowseMediaResponse, GetAlbumDetailsRequest, GetAlbumDetailsResponse,
    GetAlbumsRequest, GetAlbumsResponse, GetArtistDetailsRequest, GetArtistDetailsResponse,
    GetArtistsRequest, GetArtistsResponse, GetLikedTracksRequest, GetLikedTracksResponse,
    GetMediaAudioOptionsRequest, GetMediaAudioOptionsResponse, GetMediaBrowserRequest,
    GetMediaBrowserResponse, GetMediaContainerTracksRequest, GetMediaContainerTracksResponse,
    GetTrackDetailsRequest, GetTrackDetailsResponse, GetTracksRequest, GetTracksResponse,
    LikeTrackRequest, LikeTrackResponse, MediaBrowser, MediaEntry, ScanRequest, ScanResponse,
    SearchRequest, SearchResponse,
};

pub struct Library {
    db: Database,
    sources: SourceResolver,
    /// Where the library is read from. `None` current means local files.
    ///
    /// Shared with the GraphQL schema, so the two API surfaces cannot
    /// disagree about which server a client is looking at — which is exactly
    /// what happened while only GraphQL knew: the Slint desktop and the TUI
    /// read over gRPC and kept showing the local library after a switch.
    providers: Arc<ProviderState>,
    /// The live queue. A like must land on the queued copies too: the
    /// now-playing readout serves `liked` from them, and a heart that only
    /// reaches the remote server never changes on screen.
    tracklist: Arc<std::sync::Mutex<TracklistState>>,
}

impl Library {
    // Keep the same provider across awaits and label every result with its
    // account. A client can discard a reply to a superseded navigation.
    async fn media_provider(&self, server_id: &str) -> Result<ConnectedProvider, tonic::Status> {
        let current = self.providers.current().await.ok_or_else(|| {
            tonic::Status::failed_precondition("Connect a media server to browse its libraries")
        })?;
        if current.config.id != server_id {
            return Err(tonic::Status::failed_precondition(
                "The browsing server changed; refresh its libraries",
            ));
        }
        if !current.provider.capabilities().media_browse {
            return Err(tonic::Status::failed_precondition(
                "This server does not support media library navigation",
            ));
        }
        Ok(current)
    }

    pub fn new(
        db: Database,
        providers: Arc<ProviderState>,
        tracklist: Arc<std::sync::Mutex<TracklistState>>,
    ) -> Self {
        let sources = SourceResolver::from_settings(db.clone());
        Self {
            sources,
            db,
            providers,
            tracklist,
        }
    }
    /// Share the same account lifecycle as playback and queue validation.
    pub fn with_source_resolver(mut self, sources: SourceResolver) -> Self {
        self.sources = sources;
        self
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

impl From<audio::AudioOptions> for GetMediaAudioOptionsResponse {
    fn from(options: audio::AudioOptions) -> Self {
        Self {
            source: options.source,
            versions: options
                .versions
                .into_iter()
                .map(|version| AudioVersionOption {
                    id: version.id,
                    name: version.name,
                    runtime_ticks: version.runtime_ticks,
                    default_audio_stream_index: version.default_audio_stream_index,
                    unavailable_reason: version.unavailable_reason,
                    audio_streams: version
                        .audio_streams
                        .into_iter()
                        .map(|stream| AudioStreamOption {
                            index: stream.index,
                            title: stream.title,
                            display_title: stream.display_title,
                            language: stream.language,
                            codec: stream.codec,
                            channels: stream.channels,
                            sample_rate: stream.sample_rate,
                            is_default: stream.is_default,
                            unavailable_reason: stream.unavailable_reason,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

#[tonic::async_trait]
impl LibraryService for Library {
    async fn get_media_audio_options(
        &self,
        request: tonic::Request<GetMediaAudioOptionsRequest>,
    ) -> Result<tonic::Response<GetMediaAudioOptionsResponse>, tonic::Status> {
        let source = SourceHandle::parse(&request.into_inner().source)
            .map_err(|error| tonic::Status::invalid_argument(error.to_string()))?;
        let options = self
            .sources
            .audio_options_handle(&source)
            .await
            .map_err(provider_status)?;
        Ok(tonic::Response::new(options.into()))
    }

    async fn get_media_browser(
        &self,
        _: tonic::Request<GetMediaBrowserRequest>,
    ) -> Result<tonic::Response<GetMediaBrowserResponse>, tonic::Status> {
        let browser = self.providers.current().await.map(|current| MediaBrowser {
            server_id: current.config.id,
            supported: current.provider.capabilities().media_browse,
        });
        Ok(tonic::Response::new(GetMediaBrowserResponse { browser }))
    }

    async fn browse_media(
        &self,
        request: tonic::Request<BrowseMediaRequest>,
    ) -> Result<tonic::Response<BrowseMediaResponse>, tonic::Status> {
        let request = request.into_inner();
        let limit = request.limit.unwrap_or(100);
        if request.offset < 0 || !(1..=500).contains(&limit) {
            return Err(tonic::Status::invalid_argument(
                "Media pages need a nonnegative offset and a limit from 1 to 500",
            ));
        }
        let next = request
            .offset
            .checked_add(limit)
            .ok_or_else(|| tonic::Status::invalid_argument("Media page offset is too large"))?;
        let current = self.media_provider(&request.server_id).await?;
        let mut entries = current
            .provider
            .browse(
                request.parent.as_deref(),
                Page::new(request.offset, limit + 1),
            )
            .await
            .map_err(provider_status)?;
        let next_offset = (entries.len() > limit as usize).then_some(next);
        entries.truncate(limit as usize);
        let entries = entries
            .into_iter()
            .map(|entry| MediaEntry {
                id: entry.id,
                title: entry.title,
                item_type: entry.item_type,
                media_type: entry.media_type,
                is_container: entry.is_container,
                season_number: entry.season_number,
                episode_number: entry.episode_number,
                track: entry.track.map(|track| {
                    music_player_provider::url::decorate(track, &current.config).into()
                }),
            })
            .collect();
        Ok(tonic::Response::new(BrowseMediaResponse {
            server_id: current.config.id,
            entries,
            next_offset,
        }))
    }

    async fn get_media_container_tracks(
        &self,
        request: tonic::Request<GetMediaContainerTracksRequest>,
    ) -> Result<tonic::Response<GetMediaContainerTracksResponse>, tonic::Status> {
        let request = request.into_inner();
        if request.offset < 0 || request.parent.is_empty() {
            return Err(tonic::Status::invalid_argument(
                "A container and nonnegative offset are required",
            ));
        }
        let current = self.media_provider(&request.server_id).await?;
        let tracks = current
            .provider
            .container_tracks(
                &request.parent,
                Page::new(request.offset, request.limit.unwrap_or(100)),
            )
            .await
            .map_err(provider_status)?;
        let tracks = music_player_provider::url::decorate_all(tracks, &current.config)
            .into_iter()
            .map(Into::into)
            .collect();
        // Read-only expansion: queue changes belong to the explicit AddTracks RPC.
        Ok(tonic::Response::new(GetMediaContainerTracksResponse {
            server_id: current.config.id,
            tracks,
        }))
    }

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

        // Federated: the connected server and this machine's own index are
        // searched together, so one box covers everything reachable rather
        // than describing only whichever library happens to be current. A
        // failure on either side contributes nothing rather than failing the
        // search — half an answer beats none.
        let remote = match self.providers.current().await {
            Some(current) => match current.provider.search(&query, Page::new(0, 50)).await {
                Ok(results) => {
                    let config = &current.config;
                    SearchResponse {
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
                    }
                }
                Err(e) => {
                    tracing::warn!("searching {} failed: {e}", current.config.name);
                    SearchResponse::default()
                }
            },
            None => SearchResponse::default(),
        };

        let searcher = Searcher::new(self.db.get_connection().clone());

        let tracks = searcher.search_song(&query).await.unwrap_or_default();
        let albums = searcher.search_album(&query).await.unwrap_or_default();
        let artists = searcher.search_artist(&query).await.unwrap_or_default();

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

        // The connected server leads: it is the library the screens are
        // showing, so it is what the user is most likely looking for.
        Ok(tonic::Response::new(SearchResponse {
            tracks: [remote.tracks, response.tracks].concat(),
            albums: [remote.albums, response.albums].concat(),
            artists: [remote.artists, response.artists].concat(),
        }))
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
        let ids = music_player_storage::rocksky_likes::matched_track_ids(self.db.get_connection())
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
            self.tracklist
                .lock()
                .unwrap()
                .set_track_liked(&request.id, request.like);
            return Ok(tonic::Response::new(LikeTrackResponse {}));
        }
        self.tracklist
            .lock()
            .unwrap()
            .set_track_liked(&request.id, request.like);
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
