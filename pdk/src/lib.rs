use extism_pdk::*;
use serde::{Deserialize, Serialize};

pub mod library;
pub mod player;
pub mod types;

#[host_fn]
extern "ExtismHost" {
    fn open_media(url: String);
    fn browse(url: &str);
    fn register_addon(name: String);
    fn currently_playing_song();
    fn next();
    fn previous();
    fn pause();
    fn play();
    fn stop();
    fn seek(time: u32);
    fn position_ms() -> u32;
    fn playlist();
    fn playlists();
    fn main_playlists();
    fn recent_playlists();
    fn folder();
    fn folders();
    fn create_playlist(args: Json<CreatePlaylistOpt>);
    fn delete_playlist(id: String);
    fn add_to_playlist(args: Json<AddToPlaylistOpt>);
    fn remove_from_playlist(args: Json<RemoveFromPlaylistOpt>);
    fn rename_playlist(args: Json<RenamePlaylistOpt>);
    fn with_capabilities(capabilities: Json<Vec<String>>);
}

pub enum Capability {
    Player,
    Browse,
    Share,
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        match s {
            "player" => Capability::Player,
            "browse" => Capability::Browse,
            "share" => Capability::Share,
            _ => Capability::Player,
        }
    }
}

impl From<String> for Capability {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}

impl Capability {
    pub fn to_string(&self) -> String {
        match self {
            Capability::Player => "player".into(),
            Capability::Browse => "browse".into(),
            Capability::Share => "share".into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreatePlaylistOpt {
    pub name: String,
    pub description: String,
    pub folder_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct AddToPlaylistOpt {
    pub playlist_id: String,
    pub song_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct RemoveFromPlaylistOpt {
    pub playlist_id: String,
    pub song_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct RenamePlaylistOpt {
    pub playlist_id: String,
    pub name: String,
}

pub fn addon() -> Addon {
    Addon {}
}

pub struct Addon {}

impl Addon {
    pub fn register(&self, name: &str) -> Result<&Addon, Error> {
        unsafe { register_addon(name.into())? };
        Ok(self)
    }

    pub fn with_capabilities(&self, capabilities: Vec<Capability>) -> Result<&Addon, Error> {
        let capabilities: Vec<String> = capabilities.iter().map(|c| c.to_string()).collect();
        unsafe { with_capabilities(Json(capabilities))? };
        Ok(self)
    }
}
