use super::{Addon, StreamingAddon};

pub struct Local {
    name: String,
    version: String,
    author: String,
    description: String,
    enabled: bool,
}

impl Local {
    pub fn new() -> Self {
        Self {
            name: "Local".to_string(),
            version: "0.1.0".to_string(),
            author: "Tsiry Sandratraina".to_string(),
            description: "Local addon".to_string(),
            enabled: true,
        }
    }
}

impl Addon for Local {
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

impl StreamingAddon for Local {
    fn stream(&self, url: &str) -> Result<(), Box<dyn std::error::Error>> {
        todo!("Implement Local::stream");
    }
}
