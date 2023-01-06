use music_player_types::types::{Album as AlbumType, RemoteCoverUrl, Song, Track as TrackType};
use sea_orm::{entity::prelude::*, ActiveValue};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "album")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub title: String,
    pub artist: String,
    pub artist_id: Option<String>,
    pub year: Option<u32>,
    pub cover: Option<String>,
    #[sea_orm(ignore)]
    pub tracks: Vec<super::track::Model>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::track::Entity")]
    Track,
    #[sea_orm(
        belongs_to = "super::artist::Entity",
        from = "Column::ArtistId",
        to = "super::artist::Column::Id"
    )]
    Artist,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Track.def()
    }
}

impl Related<super::artist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Artist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<&Song> for ActiveModel {
    fn from(song: &Song) -> Self {
        let id = format!("{:x}", md5::compute(format!("{}", song.album)));
        Self {
            id: ActiveValue::set(id),
            title: ActiveValue::Set(song.album.clone()),
            artist: ActiveValue::Set(song.album_artist.clone()),
            artist_id: ActiveValue::Set(Some(format!(
                "{:x}",
                md5::compute(song.album_artist.to_owned())
            ))),
            year: ActiveValue::Set(song.year),
            cover: ActiveValue::Set(song.cover.clone()),
        }
    }
}

impl From<AlbumType> for Model {
    fn from(album: AlbumType) -> Self {
        let tracks: Vec<TrackType> = album
            .clone()
            .tracks
            .into_iter()
            .map(|track| TrackType {
                album: Some(album.clone()),
                ..track
            })
            .collect();
        Self {
            id: album.id.clone(),
            title: album.title,
            cover: album.cover,
            artist: album.artist,
            artist_id: album.artist_id,
            year: album.year,
            tracks: tracks.into_iter().map(Into::into).collect(),
        }
    }
}

impl Into<AlbumType> for Model {
    fn into(self) -> AlbumType {
        AlbumType {
            id: self.id,
            title: self.title,
            cover: self.cover,
            artist: self.artist,
            artist_id: self.artist_id,
            year: self.year,
            tracks: self.tracks.into_iter().map(Into::into).collect(),
        }
    }
}

impl RemoteCoverUrl for Model {
    fn with_remote_cover_url(&self, base_url: &str) -> Self {
        Self {
            cover: self
                .cover
                .clone()
                .map(|cover| format!("{}/covers/{}", base_url, cover)),
            ..self.clone()
        }
    }
}
