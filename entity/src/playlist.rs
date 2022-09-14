use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub name: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        super::playlist_tracks::Relation::Track.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::playlist_tracks::Relation::Playlist.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
