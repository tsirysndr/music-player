use std::time::Duration;

use super::select_result::PlaylistTrack;
use super::track::ActiveModel as Track;
use super::{
    album as album_entity, artist as artist_entity, artist_tracks as artist_track_entity,
    track as track_entity,
};
use music_player_types::types::Song;
use sea_orm::ActiveValue;

#[test]
fn song_to_track() {
    let song = Song {
        artist: "PnB Rock".to_owned(),
        title: "Face".to_owned(),
        genre: "None".to_owned(), 
        year: Some(2017),
        track: Some(3),
        bitrate: Some(96),
        sample_rate: Some(22050),
        bit_depth: None,
        channels: Some(2),
        duration: Duration::from_secs(200),
        uri: Some("/Users/tsirysandratraina/Music/PnB Rock/[E]  Catch These Vibes [81127090] [2017]/03 - PnB Rock - Face(Explicit).m4a".to_owned()),
        album: "Catch These Vibes".to_owned(),
        album_artist: "PnB Rock".to_owned(),
        ..Default::default()
    };
    let track = Track::from(&song);
    let id = format!("{:x}", md5::compute(song.uri.as_ref().unwrap()));
    assert_eq!(track.id, ActiveValue::set(id));
    assert_eq!(track.artist, ActiveValue::Set(song.artist));
    assert_eq!(track.title, ActiveValue::Set(song.title));
    assert_eq!(track.genre, ActiveValue::Set(song.genre));
    assert_eq!(track.year, ActiveValue::Set(song.year));
    assert_eq!(track.track, ActiveValue::Set(song.track));
    assert_eq!(track.bitrate, ActiveValue::Set(song.bitrate));
    assert_eq!(track.sample_rate, ActiveValue::Set(song.sample_rate));
    assert_eq!(track.bit_depth, ActiveValue::Set(song.bit_depth));
    assert_eq!(track.channels, ActiveValue::Set(song.channels));
    assert_eq!(
        track.duration,
        ActiveValue::Set(Some(song.duration.as_secs_f32()))
    );
    assert_eq!(
        track.uri,
        ActiveValue::Set(song.uri.clone().unwrap_or_default())
    );
    assert_eq!(
        track.album_id,
        ActiveValue::Set(Some(format!(
            "{:x}",
            md5::compute(format!("{}", song.album))
        )))
    );
    assert_eq!(
        track.artist_id,
        ActiveValue::Set(Some(format!(
            "{:x}",
            md5::compute(song.album_artist.to_owned())
        )))
    );
}

#[test]
fn playlist_track_to_track() {
    let playlist_track = PlaylistTrack {
        id: "claxdwayb000jc9cjtnpbqytg".to_owned(),
        name: "New South".to_owned(),
        description: Some("description".to_owned()),
        album_year: Some(2020),
        track_number: Some(2),
        track_genre: Some("None".to_owned()),
        track_id: "ea5aab8970f313005739045363f44f41".to_owned(),
        track_title: "Lo Mein".to_owned(),
        track_artist: "Lil Uzi Vert".to_owned(),
        track_uri: "".to_owned(),
        album_id: "27234641d4f5f9e0832affa79b9f62d8".to_owned(),
        album_title: "Eternal Atake".to_owned(),
        album_cover: Some("27234641d4f5f9e0832affa79b9f62d8.jpg".to_owned()),
        artist_id: "0afe1226a5a75408acb57e97bd5feca1".to_owned(),
        artist_name: "Lil Uzi Vert".to_owned(),
        track_duration: 195.41299438476562,
    };
    let track = track_entity::Model::from(playlist_track.clone());
    assert_eq!(track.id, playlist_track.track_id);
    assert_eq!(track.title, playlist_track.track_title);
    assert_eq!(track.artist, playlist_track.track_artist);
    assert_eq!(track.uri, playlist_track.track_uri);
    assert_eq!(track.album_id, Some(playlist_track.album_id.clone()));
    assert_eq!(track.artist_id, Some(playlist_track.artist_id.clone()));
    assert_eq!(track.duration, Some(playlist_track.track_duration));
    assert_eq!(track.album.id, playlist_track.album_id.clone());
    assert_eq!(track.album.title, playlist_track.album_title);
    assert_eq!(track.album.cover, playlist_track.album_cover);
    assert_eq!(track.artists[0].id, playlist_track.artist_id.clone());
    assert_eq!(track.artists[0].name, playlist_track.artist_name);
}

#[test]
fn song_to_album() {
    let song = Song {
        artist: "PnB Rock".to_owned(),
        title: "Face".to_owned(),
        genre: "None".to_owned(), 
        year: Some(2017),
        track: Some(3),
        bitrate: Some(96),
        sample_rate: Some(22050),
        bit_depth: None,
        channels: Some(2),
        duration: Duration::from_secs(200),
        uri: Some("/Users/tsirysandratraina/Music/PnB Rock/[E]  Catch These Vibes [81127090] [2017]/03 - PnB Rock - Face(Explicit).m4a".to_owned()),
        album: "Catch These Vibes".to_owned(),
        album_artist: "PnB Rock".to_owned(),
        ..Default::default()
    };
    let album = album_entity::ActiveModel::from(&song);
    let id = format!("{:x}", md5::compute(format!("{}", song.album.clone())));
    assert_eq!(album.id, ActiveValue::set(id));
    assert_eq!(album.title, ActiveValue::Set(song.album.clone()));
    assert_eq!(
        album.artist_id,
        ActiveValue::Set(Some(format!(
            "{:x}",
            md5::compute(song.album_artist.clone())
        )))
    );
    assert_eq!(album.year, ActiveValue::Set(song.year));
    assert_eq!(album.cover, ActiveValue::Set(None));
}

#[test]
fn song_to_artist_track() {
    let song = Song {
        artist: "PnB Rock".to_owned(),
        title: "Face".to_owned(),
        genre: "None".to_owned(), 
        year: Some(2017),
        track: Some(3),
        bitrate: Some(96),
        sample_rate: Some(22050),
        bit_depth: None,
        channels: Some(2),
        duration: Duration::from_secs(200),
        uri: Some("/Users/tsirysandratraina/Music/PnB Rock/[E]  Catch These Vibes [81127090] [2017]/03 - PnB Rock - Face(Explicit).m4a".to_owned()),
        album: "Catch These Vibes".to_owned(),
        album_artist: "PnB Rock".to_owned(),
        ..Default::default()
    };
    let artist_track = artist_track_entity::ActiveModel::from(&song);
    let id = format!(
        "{:x}",
        md5::compute(format!("{}{}", song.artist, song.uri.as_ref().unwrap()))
    );
    let track_id = format!("{:x}", md5::compute(song.uri.as_ref().unwrap()));
    let artist_id = format!("{:x}", md5::compute(song.album_artist.clone()));
    assert_eq!(artist_track.id, ActiveValue::set(id));
    assert_eq!(artist_track.track_id, ActiveValue::set(track_id));
    assert_eq!(artist_track.artist_id, ActiveValue::set(artist_id));
}

#[test]
fn song_to_artist() {
    let song = Song {
        artist: "PnB Rock".to_owned(),
        title: "Face".to_owned(),
        genre: "None".to_owned(), 
        year: Some(2017),
        track: Some(3),
        bitrate: Some(96),
        sample_rate: Some(22050),
        bit_depth: None,
        channels: Some(2),
        duration: Duration::from_secs(200),
        uri: Some("/Users/tsirysandratraina/Music/PnB Rock/[E]  Catch These Vibes [81127090] [2017]/03 - PnB Rock - Face(Explicit).m4a".to_owned()),
        album: "Catch These Vibes".to_owned(),
        album_artist: "PnB Rock".to_owned(),
        ..Default::default()
    };
    let artist = artist_entity::ActiveModel::from(&song);
    assert_eq!(
        artist.id,
        ActiveValue::set(format!("{:x}", md5::compute(song.album_artist.clone())))
    );
    assert_eq!(artist.name, ActiveValue::Set(song.album_artist.clone()));
}
