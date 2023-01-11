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

impl Into<PlaylistType> for Model {
    fn into(self) -> PlaylistType {
        PlaylistType {
            id: self.id,
            name: self.name,
            description: self.description,
            tracks: self.tracks.into_iter().map(Into::into).collect(),
        }
    }
}
