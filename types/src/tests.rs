use std::time::Duration;

use super::types::*;
use lofty::{Accessor, ItemKey, ItemValue, Tag, TagItem, TagType};
use tantivy::{
    schema::{Schema, SchemaBuilder, STORED, STRING, TEXT},
    Document,
};

#[test]
fn document_to_artist() {
    let mut schema_builder: SchemaBuilder = Schema::builder();

    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let name_field = schema_builder.add_text_field("name", TEXT | STORED);

    schema_builder.build();

    let mut doc = Document::default();
    doc.add_text(id_field, "id");
    doc.add_text(name_field, "name");

    let artist = Artist::from(doc);

    assert_eq!(artist.id, "id");
    assert_eq!(artist.name, "name");
}

#[test]
fn document_to_album() {
    let mut schema_builder: SchemaBuilder = Schema::builder();

    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let artist_field = schema_builder.add_text_field("artist", TEXT | STORED);
    let year_field = schema_builder.add_i64_field("year", STORED);
    let cover_field = schema_builder.add_text_field("cover", STRING | STORED);

    schema_builder.build();

    let mut doc = Document::default();
    doc.add_text(id_field, "id");
    doc.add_text(title_field, "title");
    doc.add_text(artist_field, "artist");
    doc.add_i64(year_field, 2020);
    doc.add_text(cover_field, "cover");

    let album = Album::from(doc);

    assert_eq!(album.id, "id");
    assert_eq!(album.title, "title");
    assert_eq!(album.artist, "artist");
    assert_eq!(album.year, Some(2020));
    assert_eq!(album.cover, Some("cover".to_string()));
}

#[test]
fn document_to_simplified_song() {
    let mut schema_builder: SchemaBuilder = Schema::builder();

    let id_field = schema_builder.add_text_field("id", STRING | STORED);
    let title_field = schema_builder.add_text_field("title", TEXT | STORED);
    let artist_field = schema_builder.add_text_field("artist", TEXT | STORED);
    let album_field = schema_builder.add_text_field("album", TEXT | STORED);
    let genre_field = schema_builder.add_text_field("genre", TEXT);
    let cover_field = schema_builder.add_text_field("cover", STRING | STORED);
    let duration_field = schema_builder.add_i64_field("duration", STORED);
    let artist_id = schema_builder.add_text_field("artist_id", STRING | STORED);
    let album_id = schema_builder.add_text_field("album_id", STRING | STORED);

    schema_builder.build();

    let mut doc = Document::default();
    doc.add_text(id_field, "id");
    doc.add_text(title_field, "title");
    doc.add_text(artist_field, "artist");
    doc.add_text(album_field, "album");
    doc.add_text(genre_field, "genre");
    doc.add_text(cover_field, "cover");
    doc.add_i64(duration_field, 100);
    doc.add_text(artist_id, "artist_id");
    doc.add_text(album_id, "album_id");

    let song = SimplifiedSong::from(doc);

    assert_eq!(song.id, "id");
    assert_eq!(song.title, "title");
    assert_eq!(song.artist, "artist");
    assert_eq!(song.album, "album");
    assert_eq!(song.genre, "genre");
    assert_eq!(song.cover, Some("cover".to_string()));
    assert_eq!(song.duration, Duration::from_secs(100));
    assert_eq!(song.artist_id, "artist_id");
    assert_eq!(song.album_id, "album_id");
}

#[test]
fn tag_to_artist() {
    let mut tag = Tag::new(TagType::ID3v2);

    tag.insert_item(TagItem::new(
        ItemKey::AlbumArtist,
        ItemValue::Text("J. Cole".to_owned()),
    ));

    let artist = Artist::from(&tag);

    let id = format!(
        "{:x}",
        md5::compute(
            tag.get_string(&ItemKey::AlbumArtist)
                .unwrap_or(tag.artist().unwrap_or("None"))
                .to_string()
        )
    );

    assert_eq!(artist.id, id);
    assert_eq!(artist.name, "J. Cole");
}

#[test]
fn tag_to_album() {
    let mut tag = Tag::new(TagType::ID3v2);

    tag.insert_item(TagItem::new(
        ItemKey::AlbumTitle,
        ItemValue::Text("The Off-Season".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::AlbumArtist,
        ItemValue::Text("J. Cole".to_owned()),
    ));

    let artist_id = Some(format!(
        "{:x}",
        md5::compute(
            tag.get_string(&ItemKey::AlbumArtist)
                .unwrap_or(tag.artist().unwrap_or("None"))
                .to_string()
        )
    ));

    let album = Album::from(&tag);

    let id = format!(
        "{:x}",
        md5::compute(
            tag.get_string(&ItemKey::AlbumTitle)
                .unwrap_or(tag.album().unwrap_or("None"))
                .to_string()
        )
    );

    assert_eq!(album.id, id);
    assert_eq!(album.title, "The Off-Season");
    assert_eq!(album.artist, "J. Cole");
    assert_eq!(album.artist_id, artist_id);
}

#[test]
fn tag_to_simplified_song() {
    let mut tag = Tag::new(TagType::ID3v2);

    tag.insert_item(TagItem::new(
        ItemKey::TrackTitle,
        ItemValue::Text("The Climb Back".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::TrackArtist,
        ItemValue::Text("J. Cole".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::AlbumArtist,
        ItemValue::Text("J. Cole".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::AlbumTitle,
        ItemValue::Text("The Off-Season".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::Genre,
        ItemValue::Text("Hip-Hop".to_owned()),
    ));
    tag.insert_item(TagItem::new(
        ItemKey::AlbumArtist,
        ItemValue::Text("J. Cole".to_owned()),
    ));

    let song = Song::from(&tag);

    assert_eq!(song.title, "The Climb Back");
    assert_eq!(song.artist, "J. Cole");
    assert_eq!(song.album, "The Off-Season");
    assert_eq!(song.genre, "Hip-Hop");
    assert_eq!(song.album_artist, "J. Cole");
}
