use super::Addon;

pub struct Musicbrainz {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
}

impl Musicbrainz {
    pub fn new() -> Self {
        Self {
            name: "Musicbrainz".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Musicbrainz addon".to_string(),
            enabled: true,
        }
    }
}

impl Addon for Musicbrainz {
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
