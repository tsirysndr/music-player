use sea_orm::entity::prelude::*;

/// Which extensions the user has switched off.
///
/// Only the flag is stored. Everything else about an extension — name,
/// version, capabilities, permissions — is read from its manifest on disk,
/// which is what changes when one is upgraded; a copy here would go stale.
#[derive(Clone, Debug, Default, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extension")]
pub struct Model {
    /// The manifest id, e.g. `com.example.lyrics`.
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub enabled: bool,
    pub updated_at: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
