use anyhow::Error;
use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, playlist::PlaylistClient,
    tracklist::TracklistClient,
};
use music_player_server::api::metadata::v1alpha1::{Album, Track};
use music_player_settings::{read_settings, Settings};
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
    GetPlaylists,
    PlayPlaylist(String),
    LoadSearchIndex,
    /// Fetch the next page of a paged browse collection and append it.
    LoadMore(PagedCollection),
}

pub struct Network<'a> {
    pub app: &'a Arc<Mutex<App>>,
    library: LibraryClient,
    playback: PlaybackClient,
    tracklist: TracklistClient,
    playlist: PlaylistClient,
}

impl<'a> Network<'a> {
    pub async fn new(app: &'a Arc<Mutex<App>>) -> Result<Network<'a>, Error> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let library = LibraryClient::new(settings.host.clone(), settings.port).await?;
        let playback = PlaybackClient::new(settings.host.clone(), settings.port).await?;
        let tracklist = TracklistClient::new(settings.host.clone(), settings.port).await?;
        let playlist = PlaylistClient::new(settings.host.clone(), settings.port).await?;
        Ok(Network {
            app,
            library,
            playback,
            tracklist,
            playlist,
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
            IoEvent::GetPlaylists => self.get_playlists().await,
            IoEvent::PlayPlaylist(id) => self.play_playlist(id).await,
            IoEvent::LoadSearchIndex => self.load_search_index().await,
            IoEvent::LoadMore(collection) => self.load_more(collection).await,
        }
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
        let mut app = self.app.lock().await;
        app.volume = volume.min(100);
        Ok(())
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
