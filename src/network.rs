use music_player_client::{
    library::LibraryClient, playback::PlaybackClient, tracklist::TracklistClient,
};
use music_player_server::metadata::v1alpha1::{Artist, Track};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::app::{AlbumTable, App, ArtistTable, TrackTable};

#[derive(Debug)]
pub enum IoEvent {
    PlayTrack(String),
    NextTrack,
    PreviousTrack,
    GetTracks,
    GetAlbums,
    GetAlbum(String),
    GetArtists,
    GetPlayQueue,
    GetAlbumTracks(String),
    AddItemToQueue(String),
    Shuffle(bool),
    Repeat(bool),
    GetCurrentPlayback,
    Play,
    Pause,
}

pub struct Network<'a> {
    pub app: &'a Arc<Mutex<App>>,
    library: LibraryClient,
    playback: PlaybackClient,
    tracklist: TracklistClient,
    playlist: PlaybackClient,
}

impl<'a> Network<'a> {
    pub async fn new(app: &'a Arc<Mutex<App>>) -> Result<Network<'a>, Box<dyn std::error::Error>> {
        let library = LibraryClient::new().await?;
        let playback = PlaybackClient::new().await?;
        let tracklist = TracklistClient::new().await?;
        let playlist = PlaybackClient::new().await?;
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
    ) -> Result<(), Box<dyn std::error::Error>> {
        match io_event {
            IoEvent::PlayTrack(track_id) => self.play_track(track_id).await,
            IoEvent::NextTrack => self.next_track().await,
            IoEvent::PreviousTrack => self.previous_track().await,
            IoEvent::GetTracks => self.get_tracks().await,
            IoEvent::GetAlbums => self.get_albums().await,
            IoEvent::GetAlbum(id) => self.get_album(id).await,
            IoEvent::GetArtists => self.get_artists().await,
            IoEvent::GetPlayQueue => self.get_play_queue().await,
            IoEvent::GetAlbumTracks(id) => self.get_album_tracks(id).await,
            IoEvent::AddItemToQueue(id) => self.add_item_to_queue(id).await,
            IoEvent::Shuffle(enable) => self.shuffle(enable).await,
            IoEvent::Repeat(enable) => self.repeat(enable).await,
            IoEvent::GetCurrentPlayback => self.get_current_playback().await,
            IoEvent::Play => self.play().await,
            IoEvent::Pause => self.pause().await,
        }
    }

    async fn play_track(&mut self, track_id: String) -> Result<(), Box<dyn std::error::Error>> {
        self.tracklist.add(&track_id).await?;
        Ok(())
    }

    async fn next_track(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.playback.next().await
    }

    async fn previous_track(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.playback.prev().await
    }

    async fn get_tracks(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tracks = self.library.songs().await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_albums(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let albums = self.library.albums().await?;
        let mut app = self.app.lock().await;
        app.album_table = AlbumTable {
            albums,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_album(&mut self, id: String) -> Result<(), Box<dyn std::error::Error>> {
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
                artists: t
                    .artists
                    .iter()
                    .map(|a| Artist {
                        name: a.name.clone(),
                        ..Default::default()
                    })
                    .collect(),
                ..Default::default()
            })
            .collect();
        app.track_table = TrackTable {
            tracks,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_artists(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let artists = self.library.artists().await?;
        let mut app = self.app.lock().await;
        app.artist_table = ArtistTable {
            artists,
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_play_queue(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (played_tracks, next_tracks) = self.tracklist.list().await?;
        let mut app = self.app.lock().await;
        app.track_table = TrackTable {
            tracks: [played_tracks, next_tracks].concat(),
            selected_index: 0,
        };
        Ok(())
    }

    async fn get_album_tracks(&mut self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn add_item_to_queue(&mut self, id: String) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn shuffle(&mut self, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn repeat(&mut self, enable: bool) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn get_current_playback(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn play(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }

    async fn pause(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        todo!()
    }
}
