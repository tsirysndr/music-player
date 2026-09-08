use music_player_types::types::Playlist as PlaylistType;
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[sea_orm(ignore)]
    pub tracks: Vec<super::track::Model>,
    pub folder_id: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// A smart playlist refills itself from [`Self::rsql`] instead of holding
    /// a hand-picked list. Its tracks still live in `playlist_tracks`, so
    /// everything that plays a playlist works unchanged.
    pub is_smart: bool,
    /// The RSQL filter, e.g. `genre==rock;year>2000`. Only meaningful when
    /// `is_smart`; empty matches the whole library.
    pub rsql: Option<String>,
    /// Track field to order by, or `random`.
    pub sort_by: Option<String>,
    /// `asc` or `desc`.
    pub sort_order: Option<String>,
    /// Cap on the number of tracks. None (or 0) is unlimited.
    pub max_tracks: Option<u32>,
    /// When the tracks were last regenerated, unix seconds.
    pub refreshed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Folder,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Folder => Entity::belongs_to(super::folder::Entity)
                .from(Column::FolderId)
                .to(super::folder::Column::Id)
                .into(),
        }
    }
}

impl Related<super::folder::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Folder.def()
    }
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_tracks::Relation::Track.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::playlist_tracks::Relation::Playlist.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for PlaylistType {
    fn from(val: Model) -> Self {
        PlaylistType {
            id: val.id,
            name: val.name,
            description: val.description,
            tracks: val.tracks.into_iter().map(Into::into).collect(),
        }
    }
}
