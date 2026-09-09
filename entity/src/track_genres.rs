use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A track's own genre tag, split and linked.
///
/// Precise when a file has one, and often it has none — which is why
/// [`artist_genres`](super::artist_genres) exists alongside it.
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, Default)]
#[sea_orm(table_name = "track_genres")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub track_id: String,
    pub genre_id: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

/// Derived from the pair, so linking the same two twice is one row.
pub fn id_for(track_id: &str, genre_id: &str) -> String {
    format!("{:x}", md5::compute(format!("{track_id}\0{genre_id}")))
}
