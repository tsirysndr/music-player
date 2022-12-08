use music_player_entity::album::Model;
use music_player_types::types::Album as AlbumType;

use crate::schema::objects::album::Album;

#[test]
fn model_to_album() {
    let model = Model {
        id: "0f33817640549ff886c8cb29e9db4f1e".to_string(),
        title: "HEROES & VILLAINS".to_string(),
        cover: Some("cover".to_string()),
        artist: "Metro Boomin".to_string(),
        year: Some(2022),
        tracks: vec![],
        ..Default::default()
    };
    let album: Album = model.into();
    assert_eq!(album.id, "0f33817640549ff886c8cb29e9db4f1e");
    assert_eq!(album.title, "HEROES & VILLAINS");
    assert_eq!(album.cover, Some("cover".to_string()));
    assert_eq!(album.artist, "Metro Boomin");
    assert_eq!(album.year, Some(2022));
    assert_eq!(album.tracks.len(), 0);
}

#[test]
fn type_to_album() {
    let album_type = AlbumType {
        id: "id".to_string(),
        title: "title".to_string(),
        cover: Some("cover".to_string()),
        artist: "artist".to_string(),
        year: Some(2021),
        ..Default::default()
    };
    let album: Album = album_type.into();
    assert_eq!(album.id, "id");
    assert_eq!(album.title, "title");
    assert_eq!(album.cover, Some("cover".to_string()));
    assert_eq!(album.artist, "artist");
    assert_eq!(album.year, Some(2021));
    assert_eq!(album.tracks.len(), 0);
}
