use anyhow::Error;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    servers::ServersClient, tracklist::TracklistClient,
};
use music_player_server::api::metadata::v1alpha1::{Album, Track};
use music_player_server::api::music::v1alpha1::SmartPlaylist;
use music_player_settings::{read_settings, Settings};
use serde_json::{json, Value};
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;

use crate::app::{
    AlbumTable, App, ArtistTable, CurrentlyPlaybackContext, LibraryCache, PagedCollection,
    Pagination, PlaylistItem, TrackTable, PAGE_SIZE, SEARCH_PAGE_SIZE,
};

#[derive(Debug)]
pub enum IoEvent {
    PlayTrack(String),
    NextTrack,
    PreviousTrack,
    GetTracks,
    GetAlbums,
    GetAlbum(String),
    GetArtists,
    GetArtist(String),
    GetPlayQueue,
    AddItemToQueue(String),
    GetCurrentPlayback,
    TogglePlayback,
    PlayTrackAt(usize),
    Seek(u32),
    SetVolume(u32),
    GetVolume,
    SetMute(bool),
    /// The saved servers, and which one the library is read from.
    LoadServers,
    ConnectServer(String),
    DisconnectServer,
    DeleteServer(String),
    /// The places the audio could come out.
    LoadRenderers,
    /// An empty id means "play here".
    ActivateRenderer {
        id: String,
        cast: bool,
    },
    AddServer {
        kind: String,
        name: String,
        url: String,
        username: String,
        password: String,
    },
    GetPlaylists,
    PlayPlaylist(String),
    LoadSearchIndex,
    /// Fetch the next page of a paged browse collection and append it.
    LoadMore(PagedCollection),
    /// Create a smart playlist from the form's rule.
    CreateSmartPlaylist {
        name: String,
        filter: String,
        sort_by: String,
        sort_order: String,
        limit: u32,
    },
    /// Count what a filter would match, for the form's status line.
    PreviewSmartPlaylist {
        filter: String,
        sort_by: String,
        limit: u32,
    },
}

/// Strip tonic's framing off a status message so the form shows the reason,
/// not the transport. A gRPC error reads like
/// `status: InvalidArgument, message: "unknown field 'x'", details: …`.
fn clean_status(raw: &str) -> String {
    if let Some(start) = raw.find("message: \"") {
        let rest = &raw[start + 10..];
        if let Some(end) = rest.find('"') {
            return rest[..end].to_string();
        }
    }
    raw.to_string()
}

pub struct Network<'a> {
    pub app: &'a Arc<Mutex<App>>,
    library: LibraryClient,
    playback: PlaybackClient,
    tracklist: TracklistClient,
    playlist: PlaylistClient,
    servers: ServersClient,
    /// The daemon's GraphQL endpoint. Cast devices live only there.
    graphql_url: String,
}

impl<'a> Network<'a> {
    pub async fn new(app: &'a Arc<Mutex<App>>) -> Result<Network<'a>, Error> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let library = LibraryClient::new(settings.host.clone(), settings.port).await?;
        let playback = PlaybackClient::new(settings.host.clone(), settings.port).await?;
        let tracklist = TracklistClient::new(settings.host.clone(), settings.port).await?;
        let playlist = PlaylistClient::new(settings.host.clone(), settings.port).await?;
        let servers = ServersClient::new(settings.host.clone(), settings.port).await?;
        let graphql_url = format!("http://{}:{}/graphql", settings.host, settings.http_port);
        Ok(Network {
            app,
            library,
            playback,
            tracklist,
            playlist,
            servers,
            graphql_url,
        })
    }

    pub async fn handle_network_event(&mut self, io_event: IoEvent) -> Result<(), Error> {
        match io_event {
            IoEvent::PlayTrack(track_id) => self.play_track(track_id).await,
            IoEvent::NextTrack => self.next_track().await,
            IoEvent::PreviousTrack => self.previous_track().await,
            IoEvent::GetTracks => self.get_tracks().await,
            IoEvent::GetAlbums => self.get_albums().await,
            IoEvent::GetAlbum(id) => self.get_album(id).await,
            IoEvent::GetArtists => self.get_artists().await,
            IoEvent::GetArtist(id) => self.get_artist(id).await,
            IoEvent::GetPlayQueue => self.get_play_queue().await,
            IoEvent::AddItemToQueue(id) => self.add_item_to_queue(id).await,
            IoEvent::GetCurrentPlayback => self.get_current_playback().await,
            IoEvent::TogglePlayback => self.toggle_playback().await,
            IoEvent::PlayTrackAt(index) => self.play_track_at(index).await,
            IoEvent::Seek(position_ms) => self.seek(position_ms).await,
            IoEvent::SetVolume(volume) => self.set_volume(volume).await,
            IoEvent::GetVolume => self.get_volume().await,
            IoEvent::SetMute(mute) => self.set_mute(mute).await,
            IoEvent::LoadServers => self.load_servers().await,
            IoEvent::ConnectServer(id) => self.connect_server(&id).await,
            IoEvent::DisconnectServer => self.disconnect_server().await,
            IoEvent::DeleteServer(id) => self.delete_server(&id).await,
            IoEvent::LoadRenderers => self.load_renderers().await,
            IoEvent::ActivateRenderer { id, cast } => self.activate_renderer(&id, cast).await,
            IoEvent::AddServer {
                kind,
                name,
                url,
                username,
                password,
            } => {
                self.add_server(&kind, &name, &url, &username, &password)
                    .await
            }
            IoEvent::GetPlaylists => self.get_playlists().await,
            IoEvent::PlayPlaylist(id) => self.play_playlist(id).await,
            IoEvent::LoadSearchIndex => self.load_search_index().await,
            IoEvent::LoadMore(collection) => self.load_more(collection).await,
            IoEvent::CreateSmartPlaylist {
                name,
                filter,
                sort_by,
                sort_order,
                limit,
            } => {
                self.create_smart_playlist(name, filter, sort_by, sort_order, limit)
                    .await
            }
            IoEvent::PreviewSmartPlaylist {
                filter,
                sort_by,
                limit,
            } => self.preview_smart_playlist(filter, sort_by, limit).await,
        }
    }

    /// Create a smart playlist and land on it. A filter the daemon rejects
    /// leaves the form open with the reason, so the user can fix it in place.
    async fn create_smart_playlist(
        &mut self,
        name: String,
        filter: String,
        sort_by: String,
        sort_order: String,
        limit: u32,
    ) -> Result<(), Error> {
        let smart = SmartPlaylist {
            filter,
            sort_by,
            sort_order,
            limit,
        };
        match self.playlist.create_smart(&name, smart).await {
            Ok(_) => {
                let mut app = self.app.lock().await;
                app.smart_playlist_form.submitting = false;
                app.smart_playlist_form.close();
                drop(app);
                self.get_playlists().await?;
            }
            Err(e) => {
                let mut app = self.app.lock().await;
                app.smart_playlist_form.submitting = false;
                app.smart_playlist_form.error = clean_status(&e.to_string());
            }
        }
        Ok(())
    }

    /// What the form's filter would match. An invalid filter comes back as the
    /// reason rather than as an error — it is invalid for most of the time it
    /// is being typed.
    async fn preview_smart_playlist(
        &mut self,
        filter: String,
        sort_by: String,
        limit: u32,
    ) -> Result<(), Error> {
        let smart = SmartPlaylist {
            filter,
            sort_by,
            sort_order: String::new(),
            limit,
        };
        let result = self.playlist.preview_smart(smart).await;
        let mut app = self.app.lock().await;
        app.smart_playlist_form.previewing = false;
        match result {
            Ok(preview) if preview.error.is_empty() => {
                app.smart_playlist_form.preview_count = Some(preview.count);
                app.smart_playlist_form.error.clear();
            }
            Ok(preview) => {
                app.smart_playlist_form.preview_count = None;
                app.smart_playlist_form.error = clean_status(&preview.error);
            }
            Err(e) => {
                app.smart_playlist_form.preview_count = None;
                app.smart_playlist_form.error = clean_status(&e.to_string());
            }
        }
        Ok(())
    }

    async fn play_track(&mut self, track_id: String) -> Result<(), Error> {
        self.tracklist.add(&track_id).await?;
        Ok(())
    }

    async fn next_track(&mut self) -> Result<(), Error> {
        self.playback.next().await
    }

    async fn previous_track(&mut self) -> Result<(), Error> {
        self.playback.prev().await
    }

    async fn get_tracks(&mut self) -> Result<(), Error> {
        let tracks = self.library.songs(None, 0, PAGE_SIZE as i32).await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            pagination: Pagination::after_page(tracks.len(), tracks.len()),
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_albums(&mut self) -> Result<(), Error> {
        let albums = self.library.albums(None, 0, PAGE_SIZE as i32).await?;
        let mut app = self.app.lock().await;
        app.album_table = AlbumTable {
            pagination: Pagination::after_page(albums.len(), albums.len()),
            albums,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_album(&mut self, id: String) -> Result<(), Error> {
        let album = self.library.album(&id).await?;
        let mut app = self.app.lock().await;
        let album = album.unwrap_or_default();
        let tracks = album
            .tracks
            .iter()
            .map(|t| Track {
                id: t.id.clone(),
                title: t.title.clone(),
                track_number: t.track_number,
                duration: t.duration,
                artist: album.artist.clone(),
                ..Default::default()
            })
            .collect();
        app.selected_album = Some(Album {
            id: album.id.clone(),
            title: album.title.clone(),
            artist: album.artist.clone(),
            year: album.year,
            ..Default::default()
        });
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
            pagination: Pagination::default(),
        };
        Ok(())
    }

    async fn get_artists(&mut self) -> Result<(), Error> {
        let artists = self.library.artists(None, 0, PAGE_SIZE as i32).await?;
        let mut app = self.app.lock().await;
        app.artist_table = ArtistTable {
            pagination: Pagination::after_page(artists.len(), artists.len()),
            artists,
            selected_index: 0,
        };
        Ok(())
    }

    /// Fetches the next page of a paged browse collection and appends it to
    /// the corresponding table. A short (or empty) page marks the collection
    /// as exhausted. The in-flight flag is always cleared, even on error, so
    /// a transient failure does not wedge incremental loading.
    async fn load_more(&mut self, collection: PagedCollection) -> Result<(), Error> {
        let offset = {
            let app = self.app.lock().await;
            match collection {
                PagedCollection::Tracks => app.track_table.pagination.next_offset,
                PagedCollection::Albums => app.album_table.pagination.next_offset,
                PagedCollection::Artists => app.artist_table.pagination.next_offset,
            }
        } as i32;

        match collection {
            PagedCollection::Tracks => {
                let result = self.library.songs(None, offset, PAGE_SIZE as i32).await;
                let mut app = self.app.lock().await;
                match result {
                    Ok(page) => {
                        app.track_table.pagination =
                            Pagination::after_page(offset as usize + page.len(), page.len());
                        app.track_table.tracks.extend(page);
                    }
                    Err(err) => {
                        app.track_table.pagination.loading = false;
                        return Err(err);
                    }
                }
            }
            PagedCollection::Albums => {
                let result = self.library.albums(None, offset, PAGE_SIZE as i32).await;
                let mut app = self.app.lock().await;
                match result {
                    Ok(page) => {
                        app.album_table.pagination =
                            Pagination::after_page(offset as usize + page.len(), page.len());
                        app.album_table.albums.extend(page);
                    }
                    Err(err) => {
                        app.album_table.pagination.loading = false;
                        return Err(err);
                    }
                }
            }
            PagedCollection::Artists => {
                let result = self.library.artists(None, offset, PAGE_SIZE as i32).await;
                let mut app = self.app.lock().await;
                match result {
                    Ok(page) => {
                        app.artist_table.pagination =
                            Pagination::after_page(offset as usize + page.len(), page.len());
                        app.artist_table.artists.extend(page);
                    }
                    Err(err) => {
                        app.artist_table.pagination.loading = false;
                        return Err(err);
                    }
                }
            }
        }
        Ok(())
    }

    async fn get_artist(&mut self, id: String) -> Result<(), Error> {
        let artist = self.library.artist(&id).await?;
        let mut app = self.app.lock().await;
        let artist = artist.unwrap_or_default();
        let tracks = artist
            .songs
            .iter()
            .map(|t| Track {
                id: t.id.clone(),
                title: t.title.clone(),
                track_number: t.track_number,
                duration: t.duration,
                artist: artist.name.clone(),
                album: t.album.as_ref().map(|a| Album {
                    id: a.id.clone(),
                    title: a.title.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect();
        app.selected_artist_name = Some(artist.name.clone());
        app.artist_albums = artist.albums.clone();
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
            pagination: Pagination::default(),
        };
        Ok(())
    }

    async fn get_play_queue(&mut self) -> Result<(), Error> {
        let (played_tracks, next_tracks) = self.tracklist.list().await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            tracks: [played_tracks, next_tracks].concat(),
            selected_index: 0,
            pagination: Pagination::default(),
        };
        Ok(())
    }

    async fn add_item_to_queue(&mut self, id: String) -> Result<(), Error> {
        self.tracklist.add(&id).await?;
        Ok(())
    }

    async fn get_current_playback(&mut self) -> Result<(), Error> {
        let (track, index, position_ms, is_playing) = self.playback.current().await?;
        let mut app = self.app.lock().await;
        app.instant_since_last_current_playback_poll = Instant::now();
        app.current_playback_context = Some(CurrentlyPlaybackContext {
            track,
            index,
            position_ms,
            is_playing,
        });
        app.is_fetching_current_playback = false;
        Ok(())
    }

    async fn toggle_playback(&mut self) -> Result<(), Error> {
        let (_, _, _, is_playing) = self.playback.current().await?;
        if is_playing {
            return self.playback.pause().await;
        }
        self.playback.play().await
    }

    async fn play_track_at(&mut self, index: usize) -> Result<(), Error> {
        self.tracklist.play_track_at(index).await
    }

    async fn seek(&mut self, position_ms: u32) -> Result<(), Error> {
        self.playback.seek(position_ms).await
    }

    async fn set_volume(&mut self, volume: u32) -> Result<(), Error> {
        self.playback.set_volume(volume).await
    }

    async fn get_volume(&mut self) -> Result<(), Error> {
        let volume = self.playback.get_volume().await?;
        let muted = self.playback.get_mute().await.unwrap_or(false);
        let mut app = self.app.lock().await;
        app.volume = volume.min(100);
        app.muted = muted;
        Ok(())
    }

    async fn set_mute(&mut self, mute: bool) -> Result<(), Error> {
        self.playback.set_mute(mute).await
    }

    // ── Servers (where the library is read from) ────────────────────────────

    async fn load_servers(&mut self) -> Result<(), Error> {
        let servers = self.servers.list().await?;
        let kinds = self.servers.source_kinds().await.unwrap_or_default();
        let mut app = self.app.lock().await;
        app.switcher.kinds = kinds
            .into_iter()
            .map(|kind| (kind.kind, kind.display_name))
            .collect();
        app.switcher.set_servers(
            servers
                .into_iter()
                .map(|server| crate::server_switcher::ServerEntry {
                    id: server.id,
                    kind: server.kind,
                    name: server.name,
                    url: server.url,
                    connected: server.connected,
                })
                .collect(),
        );
        Ok(())
    }

    /// Repoint the library at a server. Every screen has to be re-read; what
    /// is playing keeps playing.
    async fn connect_server(&mut self, id: &str) -> Result<(), Error> {
        if let Err(e) = self.servers.connect(id).await {
            let mut app = self.app.lock().await;
            app.switcher.error = clean_status(&e.to_string());
            return Ok(());
        }
        self.reload_library().await
    }

    async fn disconnect_server(&mut self) -> Result<(), Error> {
        self.servers.disconnect().await?;
        self.reload_library().await
    }

    /// Deleting the connected one drops back to the local library, so the
    /// screens are re-read either way.
    async fn delete_server(&mut self, id: &str) -> Result<(), Error> {
        self.servers.delete(id).await?;
        self.reload_library().await
    }

    async fn add_server(
        &mut self,
        kind: &str,
        name: &str,
        url: &str,
        username: &str,
        password: &str,
    ) -> Result<(), Error> {
        let name = if name.trim().is_empty() { url } else { name };
        match self.servers.add(kind, name, url, username, password).await {
            Ok(server) => {
                // Saving one is only ever a step towards using it.
                let id = server.id.clone();
                {
                    let mut app = self.app.lock().await;
                    app.switcher.form = Default::default();
                }
                self.connect_server(&id).await?;
                self.load_servers().await
            }
            Err(e) => {
                let mut app = self.app.lock().await;
                app.switcher.form.submitting = false;
                app.switcher.form.error = clean_status(&e.to_string());
                Ok(())
            }
        }
    }

    // ── Renderers (where the audio comes out) ──────────────────────────────

    /// The daemon does the discovering — Chromecast, UPnP/DLNA and mDNS peers
    /// all arrive through it — so this is a read rather than a scan.
    ///
    /// Two calls, not one: `connectedCastDevice` *errors* when nothing is
    /// connected rather than returning null, and that is the ordinary case.
    async fn load_renderers(&mut self) -> Result<(), Error> {
        const LIST: &str = r#"query { listCastDevices { id name app } }"#;
        const CONNECTED: &str = r#"query { connectedCastDevice { id } }"#;

        let data = match self.graphql(LIST, json!({})).await {
            Ok(data) => data,
            Err(e) => {
                let mut app = self.app.lock().await;
                app.renderers.loading = false;
                app.renderers.error = e.to_string();
                return Ok(());
            }
        };
        let current = self
            .graphql(CONNECTED, json!({}))
            .await
            .ok()
            .and_then(|data| {
                data["connectedCastDevice"]["id"]
                    .as_str()
                    .map(str::to_owned)
            })
            .unwrap_or_default();

        let renderers = data["listCastDevices"]
            .as_array()
            .map(|rows| {
                rows.iter()
                    .map(|row| {
                        let id = row["id"].as_str().unwrap_or_default().to_string();
                        let kind = row["app"].as_str().unwrap_or_default().to_string();
                        crate::renderer_picker::RendererEntry {
                            playing: id == current,
                            // A peer daemon is reached over the device API,
                            // everything else over the cast one.
                            cast: kind != "music-player",
                            name: row["name"].as_str().unwrap_or_default().to_string(),
                            id,
                            kind,
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut app = self.app.lock().await;
        app.renderers.error.clear();
        app.renderers.set_renderers(renderers);
        Ok(())
    }

    async fn activate_renderer(&mut self, id: &str, cast: bool) -> Result<(), Error> {
        let result = if id.is_empty() {
            const OFF: &str = r#"mutation { disconnectFromCastDevice { id } }"#;
            // Failing here just means nothing was connected.
            let _ = self.graphql(OFF, json!({})).await;
            Ok(json!({}))
        } else if cast {
            const ON: &str = r#"mutation($id: ID!) { connectToCastDevice(id: $id) { id } }"#;
            self.graphql(ON, json!({ "id": id })).await
        } else {
            const ON: &str = r#"mutation($id: ID!) { connectToDevice(id: $id) { id } }"#;
            self.graphql(ON, json!({ "id": id })).await
        };
        if let Err(e) = result {
            let mut app = self.app.lock().await;
            app.renderers.error = e.to_string();
            return Ok(());
        }
        self.load_renderers().await
    }

    /// One GraphQL round trip to the daemon.
    ///
    /// Cast devices are only exposed over GraphQL — the gRPC surface has no
    /// equivalent — so this is how the TUI reaches them. GraphQL answers 200
    /// even when it failed, so the errors array is the only thing that says so.
    async fn graphql(&self, query: &str, variables: Value) -> Result<Value, Error> {
        let response = reqwest::Client::new()
            .post(&self.graphql_url)
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .await?;
        let body: Value = response.json().await?;
        if let Some(first) = body
            .get("errors")
            .and_then(|errors| errors.as_array())
            .and_then(|errors| errors.first())
        {
            let message = first
                .get("message")
                .and_then(|message| message.as_str())
                .unwrap_or("the daemon refused the request");
            return Err(Error::msg(message.to_string()));
        }
        body.get("data")
            .cloned()
            .ok_or_else(|| Error::msg("the daemon returned no data"))
    }

    /// Everything on screen came from the old server, so it all goes.
    async fn reload_library(&mut self) -> Result<(), Error> {
        {
            let mut app = self.app.lock().await;
            app.switcher.error.clear();
            app.library_cache = None;
        }
        self.load_servers().await?;
        self.get_tracks().await?;
        self.get_playlists().await
    }

    async fn get_playlists(&mut self) -> Result<(), Error> {
        let playlists = self.playlist.list_all().await?;
        let mut app = self.app.lock().await;
        app.playlists = playlists
            .into_iter()
            .map(|p| PlaylistItem {
                id: p.id,
                name: p.name,
            })
            .collect();
        if app.selected_playlist_index.is_none() && !app.playlists.is_empty() {
            app.selected_playlist_index = Some(0);
        }
        Ok(())
    }

    async fn play_playlist(&mut self, id: String) -> Result<(), Error> {
        let playlist = self.playlist.find(&id).await?;
        if playlist.tracks.is_empty() {
            return Ok(());
        }
        self.tracklist.load_tracks(playlist.tracks, 0).await
    }

    /// Builds the fuzzy-search index by streaming the whole library in pages
    /// of [`SEARCH_PAGE_SIZE`], so arbitrarily large libraries are covered
    /// without one huge request. The cache is updated after every page (the
    /// app lock is only held briefly each time), which keeps the overlay
    /// responsive and lets results re-rank as more of the library arrives.
    async fn load_search_index(&mut self) -> Result<(), Error> {
        {
            let mut app = self.app.lock().await;
            app.library_cache = Some(LibraryCache::default());
        }

        let result = self.stream_search_index().await;

        let mut app = self.app.lock().await;
        app.search.loading = false;
        if app.search.active {
            app.refresh_search_results();
        }
        result
    }

    /// Appends one page after another to the shared library cache. A short
    /// page ends the loop for the collection being streamed.
    async fn stream_search_index(&mut self) -> Result<(), Error> {
        let page_size = SEARCH_PAGE_SIZE as i32;

        let mut offset = 0;
        loop {
            let page = self.library.songs(None, offset, page_size).await?;
            let page_len = page.len();
            offset += page_len as i32;
            let mut app = self.app.lock().await;
            if let Some(cache) = app.library_cache.as_mut() {
                cache.tracks.extend(page);
            }
            if app.search.active {
                app.refresh_search_results();
            }
            drop(app);
            if page_len < SEARCH_PAGE_SIZE {
                break;
            }
        }

        let mut offset = 0;
        loop {
            let page = self.library.albums(None, offset, page_size).await?;
            let page_len = page.len();
            offset += page_len as i32;
            let mut app = self.app.lock().await;
            if let Some(cache) = app.library_cache.as_mut() {
                cache.albums.extend(page);
            }
            if app.search.active {
                app.refresh_search_results();
            }
            drop(app);
            if page_len < SEARCH_PAGE_SIZE {
                break;
            }
        }

        let mut offset = 0;
        loop {
            let page = self.library.artists(None, offset, page_size).await?;
            let page_len = page.len();
            offset += page_len as i32;
            let mut app = self.app.lock().await;
            if let Some(cache) = app.library_cache.as_mut() {
                cache.artists.extend(page);
            }
            if app.search.active {
                app.refresh_search_results();
            }
            drop(app);
            if page_len < SEARCH_PAGE_SIZE {
                break;
            }
        }

        Ok(())
    }
}
