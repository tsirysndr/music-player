use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "playlist_track")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub playlist_id: String,
    pub track_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::playlist::Entity",
        from = "Column::PlaylistId",
        to = "super::playlist::Column::Id"
    )]
    Playlist,
    #[sea_orm(
        belongs_to = "super::track::Entity",
        from = "Column::TrackId",
        to = "super::track::Column::Id"
    )]
    Track,
}

impl ActiveModelBehavior for ActiveModel {}
