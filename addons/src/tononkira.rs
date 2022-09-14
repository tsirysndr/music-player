use super::{Addon, LyricsAddon};

pub struct Tononkira {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
}

impl Tononkira {
    pub fn new() -> Self {
        Self {
            name: "Tononkira".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Tononkira addon".to_string(),
            enabled: true,
        }
    }
}

impl Addon for Tononkira {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        &self.version
    }

    fn author(&self) -> &str {
        &self.author
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl LyricsAddon for Tononkira {
    fn get_lyrics(&self, artist: &str, title: &str) -> Option<String> {
        todo!("Implement get_lyrics for Tononkira")
    }
}
