use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// A like from the user's atproto repo (`app.rocksky.like`), flattened with the
/// `app.rocksky.song` it points at.
///
/// Likes are kept in their own table rather than as a flag on `track` so that a
/// like whose song is not in the library yet survives — and gets re-matched
/// after the next scan. `track_id` is the local track it resolved to, null
/// while unmatched.
#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rocksky_like")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub uri: String,
    pub song_uri: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub album_artist: String,
    pub created_at: String,
    pub track_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
