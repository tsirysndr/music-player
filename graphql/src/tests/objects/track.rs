use async_graphql::ID;
use music_player_entity::{select_result, track::Model};
use music_player_types::types::SimplifiedSong as TrackType;

use crate::schema::objects::track::{Track, TrackInput};

#[test]
fn model_to_track() {
    let model = Model {
        id: "0f567648e4fdea3ac86cf12e3a0c0408".to_string(),
        title: "Niagara Falls (Foot or 2)".to_string(),
        artist: "Metro Boomin, Travis Scott, 21 Savage".to_string(),
        year: Some(2022),
        ..Default::default()
    };
    let track: Track = model.into();
    assert_eq!(track.id, "0f567648e4fdea3ac86cf12e3a0c0408");
    assert_eq!(track.title, "Niagara Falls (Foot or 2)");
    assert_eq!(track.artist, "Metro Boomin, Travis Scott, 21 Savage");
}

#[test]
fn track_input_into_model() {
    let track_input = TrackInput {
        id: ID("0f567648e4fdea3ac86cf12e3a0c0408".to_string()),
        title: "Niagara Falls (Foot or 2)".to_string(),
        ..Default::default()
    };
    let model: Model = track_input.into();
    assert_eq!(model.id, "0f567648e4fdea3ac86cf12e3a0c0408");
    assert_eq!(model.title, "Niagara Falls (Foot or 2)");
}

#[test]
fn type_to_track() {
    let track_type = TrackType {
        id: "0f567648e4fdea3ac86cf12e3a0c0408".to_string(),
        title: "Niagara Falls (Foot or 2)".to_string(),
        artist: "Metro Boomin, Travis Scott, 21 Savage".to_string(),
        ..Default::default()
    };
    let track: Track = track_type.into();
    assert_eq!(track.id, "0f567648e4fdea3ac86cf12e3a0c0408");
    assert_eq!(track.title, "Niagara Falls (Foot or 2)");
    assert_eq!(track.artist, "Metro Boomin, Travis Scott, 21 Savage");
}

#[test]
fn playlist_track_to_track() {
    let playlist_track = select_result::PlaylistTrack {
        id: "claoc1y200003wvkbkjitga75".to_string(),
        name: "New South".to_string(),
        description: None,
        album_id: "0f33817640549ff886c8cb29e9db4f1e".to_string(),
        album_title: "HEROES & VILLAINS".to_string(),
        album_cover: Some("album_cover".to_string()),
        album_year: Some(2022),
        artist_id: "df2ac64a1a2c4c4b604d02b6d2d0477a".to_string(),
        artist_name: "Metro Boomin, Travis Scott, Future".to_string(),
        track_id: "36d052a19ff3535bf32dab9727410d18".to_string(),
        track_title: "Lock On Me".to_string(),
        track_duration: 174.8209991455078,
        track_number: Some(13),
        track_artist: "Metro Boomin, Travis Scott, Future".to_string(),
        track_genre: None,
        track_uri: "/Users/tsirysandratraina/Music/Metro Boomin/[E]  HEROES & VILLAINS [263828929] [2022]/13 - Metro Boomin - Lock On Me(Explicit).m4a".to_string(),
    };
    let track: Track = playlist_track.into();
    assert_eq!(track.id, ID("36d052a19ff3535bf32dab9727410d18".to_owned()));
    assert_eq!(track.title, "Lock On Me");
    assert_eq!(track.artist, "Metro Boomin, Travis Scott, Future");
    assert_eq!(
        track.album_id,
        "0f33817640549ff886c8cb29e9db4f1e".to_owned()
    );
    assert_eq!(
        track.artist_id,
        "df2ac64a1a2c4c4b604d02b6d2d0477a".to_owned()
    );
    assert_eq!(track.album_title, "HEROES & VILLAINS".to_owned());
    assert_eq!(track.album.year, Some(2022));
    assert_eq!(track.duration, Some(174.8209991455078));
    assert_eq!(track.track_number, Some(13));
}
