use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, tracklist::TracklistClient,
    ws_client::WebsocketClient,
};
use music_player_server::api::metadata::v1alpha1::{Album, Track};
use music_player_settings::{read_settings, Settings};
use std::{sync::Arc, time::Instant};
use tokio::sync::Mutex;
use anyhow::Error;

use crate::app::{AlbumTable, App, ArtistTable, CurrentlyPlaybackContext, TrackTable};

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
    GetAlbumTracks(String),
    AddItemToQueue(String),
    Shuffle(bool),
    Repeat(bool),
    GetCurrentPlayback,
    TogglePlayback,
    PlayTrackAt(usize),
}

pub struct Network<'a> {
    pub app: &'a Arc<Mutex<App>>,
    library: LibraryClient,
    playback: PlaybackClient,
    tracklist: TracklistClient,
    playlist: PlaybackClient,
}

impl<'a> Network<'a> {
    pub async fn new(app: &'a Arc<Mutex<App>>) -> Result<Network<'a>, Error> {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let library = LibraryClient::new(settings.host.clone(), settings.port).await?;
        let playback = PlaybackClient::new(settings.host.clone(), settings.port).await?;
        let tracklist = TracklistClient::new(settings.host.clone(), settings.port).await?;
        let playlist = PlaybackClient::new(settings.host.clone(), settings.port).await?;
        let _ws = WebsocketClient::new().await;
        Ok(Network {
            app,
            library,
            playback,
            tracklist,
            playlist,
        })
    }
    pub async fn handle_network_event(
        &mut self,
        io_event: IoEvent,
    ) -> Result<(), Error> {
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
            IoEvent::GetAlbumTracks(id) => self.get_album_tracks(id).await,
            IoEvent::AddItemToQueue(id) => self.add_item_to_queue(id).await,
            IoEvent::Shuffle(enable) => self.shuffle(enable).await,
            IoEvent::Repeat(enable) => self.repeat(enable).await,
            IoEvent::GetCurrentPlayback => self.get_current_playback().await,
            IoEvent::TogglePlayback => self.toggle_playback().await,
            IoEvent::PlayTrackAt(index) => self.play_track_at(index).await,
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
        let tracks = self.library.songs().await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_albums(&mut self) -> Result<(), Error> {
        let albums = self.library.albums().await?;
        let mut app = self.app.lock().await;
        app.album_table = AlbumTable {
            albums,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_album(&mut self, id: String) -> Result<(), Error> {
        let album = self.library.album(&id).await?;
        let mut app = self.app.lock().await;
        let tracks = album
            .unwrap_or_default()
            .tracks
            .iter()
            .map(|t| Track {
                id: t.id.clone(),
                title: t.title.clone(),
                track_number: t.track_number,
                duration: t.duration,
                ..Default::default()
            })
            .collect();
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_artists(&mut self) -> Result<(), Error> {
        let artists = self.library.artists().await?;
        let mut app = self.app.lock().await;
        app.artist_table = ArtistTable {
            artists,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_artist(&mut self, id: String) -> Result<(), Error> {
        let artist = self.library.artist(&id).await?;
        let mut app = self.app.lock().await;
        let tracks = artist
            .unwrap_or_default()
            .songs
            .iter()
            .map(|t| Track {
                id: t.id.clone(),
                title: t.title.clone(),
                track_number: t.track_number,
                duration: t.duration,
                album: t.album.as_ref().map(|a| Album {
                    id: a.id.clone(),
                    title: a.title.clone(),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .collect();
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_play_queue(&mut self) -> Result<(), Error> {
        let (played_tracks, next_tracks) = self.tracklist.list().await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            tracks: [played_tracks, next_tracks].concat(),
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_album_tracks(&mut self, id: String) -> Result<(), Error> {
        todo!()
    }

    async fn add_item_to_queue(&mut self, id: String) -> Result<(), Error> {
        todo!()
    }

    async fn shuffle(&mut self, enable: bool) -> Result<(), Error> {
        todo!()
    }

    async fn repeat(&mut self, enable: bool) -> Result<(), Error> {
        todo!()
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
}
