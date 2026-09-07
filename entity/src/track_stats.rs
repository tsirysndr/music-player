//! Per-track play statistics.
//!
//! Kept out of `track` so a rescan — which rewrites track rows — cannot lose
//! them, and so a track that has never been played simply has no row (the RSQL
//! `playcount` field coalesces the absence to zero).
//!
//! Timestamps are unix seconds, not datetimes: RSQL resolves relative ages like
//! `30d` into an integer, and comparing that against a text date would silently
//! match nothing.

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "track_stats")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub track_id: String,
    pub play_count: i32,
    pub skip_count: i32,
    pub last_played: Option<i64>,
    pub last_skipped: Option<i64>,
    pub updated_at: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::track::Entity",
        from = "Column::TrackId",
        to = "super::track::Column::Id"
    )]
    Track,
}

impl Related<super::track::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Track.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
