use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A genre an artist has been tagged with, from the Rocksky enrichment.
///
/// Broader coverage than the per-file tag — most files ship no usable genre —
/// at the cost of describing the artist rather than the track.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "artist_genres")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub artist_id: String,
    pub genre_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

pub fn id_for(artist_id: &str, genre_id: &str) -> String {
    format!("{:x}", md5::compute(format!("{artist_id}\0{genre_id}")))
}
