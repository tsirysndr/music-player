use music_player_entity::artist::Model;
use music_player_types::types::Artist as ArtistType;

use crate::schema::objects::artist::Artist;

#[test]
fn model_to_artist() {
    let model = Model {
        id: "df2ac64a1a2c4c4b604d02b6d2d0477a".to_string(),
        name: "Travis Scott".to_string(),
        ..Default::default()
    };
    let artist: Artist = model.into();
    assert_eq!(artist.id, "df2ac64a1a2c4c4b604d02b6d2d0477a");
    assert_eq!(artist.name, "Travis Scott");
}

#[test]
fn type_to_artist() {
    let artist_type = ArtistType {
        id: "df2ac64a1a2c4c4b604d02b6d2d0477a".to_string(),
        name: "Travis Scott".to_string(),
        ..Default::default()
    };
    let artist: Artist = artist_type.into();
    assert_eq!(artist.id, "df2ac64a1a2c4c4b604d02b6d2d0477a");
    assert_eq!(artist.name, "Travis Scott");
}
