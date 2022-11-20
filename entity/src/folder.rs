use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "folder")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    #[sea_orm(ignore)]
    pub playlists: Vec<super::playlist::Model>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Playlist,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Playlist => Entity::has_many(super::playlist::Entity).into(),
        }
    }
}

impl Related<super::playlist::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Playlist.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
