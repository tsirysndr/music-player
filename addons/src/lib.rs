pub trait Addon {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn author(&self) -> &str;
    fn description(&self) -> &str;
    fn enabled(&self) -> bool;
    fn set_enabled(&mut self, enabled: bool);
}

pub trait StreamingAddon {
    fn stream(&self, url: &str) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait LyricsAddon {
    fn get_lyrics(&self, artist: &str, title: &str) -> Option<String>;
}
