use crate::source_resolver::SourceResolver;
use music_player_settings::EmbyRuntimeSettings;
use music_player_storage::Database;

impl SourceResolver {
    /// All daemon/API entry points use the same redirect policy and device ID.
    pub fn from_settings(db: Database) -> Self {
        let settings = EmbyRuntimeSettings::read();
        Self::new(db, settings.device_id, settings.follow_redirects)
    }
}

impl SourceResolver {
    /// Construct once inside the daemon runtime, then clone for Player and APIs.
    pub fn for_daemon(db: Database) -> Result<Self, anyhow::Error> {
        use music_player_extensions::{search_paths, HostContext, Library};
        use music_player_settings::{get_application_directory, read_settings, Settings};

        let settings = read_settings()?.try_deserialize::<Settings>()?;
        let paths = search_paths(&get_application_directory(), &settings.extension_paths);
        let context = HostContext {
            library: Some(std::sync::Arc::new(Library::new(
                db.get_connection().clone(),
            )?)),
        };
        let accounts =
            crate::extension_accounts::ExtensionAccounts::new(db.clone(), paths, context);
        // MP3 has an exercised forward-only decode path. Container/file support
        // does not establish streaming support (in particular MP4 needs seeks).
        // Expand negotiation when additional stream formats have CI fixtures.
        Ok(Self::from_settings(db).with_extensions(accounts, vec!["mp3".to_owned()]))
    }
}
