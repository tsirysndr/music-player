use sea_orm::entity::prelude::*;

/// One configured source account, independent of the installed plugin instance.
#[derive(Clone, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "extension_source_account")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub extension_id: String,
    /// Fixed when created. An installed plugin upgrade cannot reinterpret old references.
    pub source_api_version: i32,
    pub name: String,
    /// Canonical JSON map of declared configuration overrides. May contain secrets.
    pub config_json: String,
    /// Changes only when configuration changes; used to reject stale bindings.
    pub config_revision: i64,
    pub remote_server_id: Option<String>,
    pub remote_user_id: Option<String>,
    /// V1 binds only the local configuration revision; it has no remote identity API.
    pub local_bound_revision: Option<i64>,
    /// Actual merged v1 config. Private credential-bearing comparison data.
    pub local_config_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionSourceAccount")
            .field("id", &self.id)
            .field("extension_id", &self.extension_id)
            .field("source_api_version", &self.source_api_version)
            .field("config_revision", &self.config_revision)
            .field("config_json", &"[redacted]")
            .finish_non_exhaustive()
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
