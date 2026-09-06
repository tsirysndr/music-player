use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// When an account's repo CAR archive was last downloaded successfully, so a
/// restart does not pull the whole repo again.
#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "atproto_repo_sync")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub did: String,
    /// RFC 3339 timestamp of the last successful download.
    pub last_downloaded_at: String,
    /// Size of that archive, kept for the logs.
    pub bytes: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
