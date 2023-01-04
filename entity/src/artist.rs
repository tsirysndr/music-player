use music_player_types::types::{Artist as ArtistType, RemoteTrackUrl, Song, RemoteCoverUrl};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "artist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(ignore)]
    pub albums: Vec<super::album::Model>,
    #[sea_orm(ignore)]
    pub tracks: Vec<super::track::Model>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::album::Entity")]
    Album,
    #[sea_orm(has_many = "super::track::Entity")]
    Track,
}

impl Related<super::album::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Album.def()
    }
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Track.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&Song> for ActiveModel {
    fn from(song: &Song) -> Self {
        let id = format!("{:x}", md5::compute(song.album_artist.to_owned()));
        Self {
            id: ActiveValue::set(id),
            name: ActiveValue::Set(song.album_artist.clone()),
        }
    }
}

impl From<ArtistType> for Model {
    fn from(artist: ArtistType) -> Self {
        Self {
            id: artist.id.clone(),
            name: artist.name,
            ..Default::default()
        }
    }
}

impl Into<ArtistType> for Model {
    fn into(self) -> ArtistType {
        ArtistType {
            id: self.id,
            name: self.name,
            ..Default::default()
        }
    }
}

impl RemoteCoverUrl for Model {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            albums: self
                .albums
                .iter()
                .map(|album| album.with_remote_cover_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}

impl RemoteTrackUrl for Model {
    fn with_remote_track_url(&self, base_url: &str) -> Self {
        Self {
            tracks: self
                .tracks
                .iter()
                .map(|track| track.with_remote_track_url(base_url))
                .collect(),
            ..self.clone()
        }
    }
}
