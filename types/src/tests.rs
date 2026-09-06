use super::types::*;
use rockbox_metadata::Metadata;

fn metadata() -> Metadata {
    Metadata {
        title: "The Climb Back".to_string(),
        artist: "J. Cole".to_string(),
        albumartist: "J. Cole".to_string(),
        album: "The Off-Season".to_string(),
        genre: "Hip-Hop".to_string(),
        ..Default::default()
    }
}

#[test]
fn metadata_to_artist() {
    let artist = Artist::from(&metadata());

    assert_eq!(artist.id, format!("{:x}", md5::compute("J. Cole")));
    assert_eq!(artist.name, "J. Cole");
}

#[test]
fn metadata_to_album() {
    let album = Album::from(&metadata());

    assert_eq!(album.id, album_id("The Off-Season", "J. Cole"));
    assert_eq!(album.title, "The Off-Season");
    assert_eq!(album.artist, "J. Cole");
    assert_eq!(
        album.artist_id,
        Some(format!("{:x}", md5::compute("J. Cole")))
    );
}

#[test]
fn same_album_title_by_different_artists_has_distinct_ids() {
    assert_ne!(
        album_id("Greatest Hits", "Artist A"),
        album_id("Greatest Hits", "Artist B")
    );
}

#[test]
fn metadata_to_song() {
    let song = Song::from(&metadata());

    assert_eq!(song.title, "The Climb Back");
    assert_eq!(song.artist, "J. Cole");
    assert_eq!(song.album, "The Off-Season");
    assert_eq!(song.genre, "Hip-Hop");
    assert_eq!(song.album_artist, "J. Cole");
}

#[test]
fn metadata_missing_tags_default_to_none() {
    let song = Song::from(&Metadata::default());

    assert_eq!(song.title, "None");
    assert_eq!(song.artist, "None");
    assert_eq!(song.album, "None");
    assert_eq!(song.album_artist, "None");
    assert_eq!(song.bitrate, None);
    assert_eq!(song.sample_rate, None);
}
