use music_player_entity::{playlist::Model, select_result};

use crate::schema::objects::playlist::Playlist;

#[test]
fn model_to_playlist() {
    let model = Model {
        id: "claoc1y200003wvkbkjitga75".to_string(),
        name: "New South".to_string(),
        description: None,
        ..Default::default()
    };
    let playlist: Playlist = model.into();
    assert_eq!(playlist.id, "claoc1y200003wvkbkjitga75");
    assert_eq!(playlist.name, "New South");
    assert_eq!(playlist.description, None);
    assert_eq!(playlist.tracks.len(), 0);
}

#[test]
fn playlist_tracks_to_playlist() {
    let tracks: Vec<select_result::PlaylistTrack> = vec![];
    let playlist: Playlist = tracks.into();
    assert_eq!(playlist.tracks.len(), 0);
}
