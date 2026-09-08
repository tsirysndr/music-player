//! How a source is addressed, independent of how it was discovered.
//!
//! A saved row in the database, an mDNS peer and a server configured in
//! `settings.toml` all reduce to the same five fields. Collapsing them here is
//! what stops the connect path from re-deriving a base url three different
//! ways — the bug that used to leave `base_url` as `None` for a music-player
//! peer and panic the first resolver that unwrapped it.

use music_player_types::types::Device;

/// Everything needed to reach a server. `url` is non-empty and absolute.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ProviderConfig {
    /// The saved-server row id, or the device id for a discovered peer.
    pub id: String,
    pub kind: String,
    pub name: String,
    /// Absolute, scheme included, no trailing slash.
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl ProviderConfig {
    /// Stable across restarts and idempotent on re-add: the same server added
    /// twice is one row.
    pub fn id_for(kind: &str, url: &str) -> String {
        format!("{:x}", md5::compute(format!("{kind}\0{}", normalize(url))))
    }

    pub fn new(kind: impl Into<String>, name: impl Into<String>, url: &str) -> Self {
        let kind = kind.into();
        let url = normalize(url);
        Self {
            id: Self::id_for(&kind, &url),
            kind,
            name: name.into(),
            url,
            username: None,
            password: None,
        }
    }

    pub fn with_credentials(mut self, username: Option<String>, password: Option<String>) -> Self {
        // Empty strings are how a form says "no credentials"; normalising here
        // keeps every backend from having to check twice.
        self.username = username.filter(|value| !value.is_empty());
        self.password = password.filter(|value| !value.is_empty());
        self
    }

    /// A discovered device, as something connectable.
    ///
    /// `http_port` is the port the *web* endpoint is on, which for a
    /// music-player peer is not the port mDNS advertised for gRPC. Callers
    /// that know it pass it; otherwise the device's own port is used.
    pub fn from_device(device: &Device, http_port: Option<u16>) -> Option<Self> {
        if device.host.is_empty() && device.base_url.is_none() {
            return None;
        }
        let url = match device.base_url.as_deref() {
            Some(base) if !base.is_empty() => normalize(base),
            _ => format!(
                "http://{}:{}",
                device.host,
                http_port.unwrap_or(device.port)
            ),
        };
        let kind = if device.app.is_empty() {
            "music-player".to_string()
        } else {
            device.app.clone()
        };
        Some(Self {
            id: device.id.clone(),
            kind,
            name: if device.name.is_empty() {
                device.host.clone()
            } else {
                device.name.clone()
            },
            url,
            username: None,
            password: None,
        })
    }

    /// Back to a `Device`, so the existing connected/disconnected broadcasts
    /// and the device list keep working unchanged.
    pub fn to_device(&self) -> Device {
        let (host, port) = self.host_port();
        Device {
            id: self.id.clone(),
            name: self.name.clone(),
            host: host.clone(),
            ip: host,
            port,
            service: self.kind.clone(),
            app: self.kind.clone(),
            is_connected: true,
            base_url: Some(self.url.clone()),
            is_cast_device: false,
            is_source_device: true,
            is_current_device: false,
        }
    }

    pub fn host_port(&self) -> (String, u16) {
        match url::Url::parse(&self.url) {
            Ok(parsed) => (
                parsed.host_str().unwrap_or_default().to_string(),
                parsed.port_or_known_default().unwrap_or(80),
            ),
            Err(_) => (String::new(), 80),
        }
    }

    pub fn host(&self) -> String {
        self.host_port().0
    }
}

/// Trailing slashes make `{base}/rest/ping` come out as `//rest/ping`, which
/// some servers accept and some do not.
fn normalize(url: &str) -> String {
    url.trim().trim_end_matches('/').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_id_survives_a_trailing_slash() {
        let bare = ProviderConfig::new("subsonic", "NAS", "http://nas.lan:4533");
        let slashed = ProviderConfig::new("subsonic", "NAS", "http://nas.lan:4533/");
        assert_eq!(
            bare.id, slashed.id,
            "the same server added twice is one row"
        );
        assert_eq!(slashed.url, "http://nas.lan:4533");
    }

    #[test]
    fn the_kind_is_part_of_the_id() {
        let subsonic = ProviderConfig::new("subsonic", "x", "http://host:8096");
        let jellyfin = ProviderConfig::new("jellyfin", "x", "http://host:8096");
        assert_ne!(subsonic.id, jellyfin.id);
    }

    #[test]
    fn empty_credentials_read_as_none() {
        let config = ProviderConfig::new("subsonic", "x", "http://h:1")
            .with_credentials(Some(String::new()), Some("pw".into()));
        assert_eq!(config.username, None);
        assert_eq!(config.password.as_deref(), Some("pw"));
    }

    /// A music-player peer is advertised over mDNS with no `base_url` at all.
    /// This is the case that used to leave a `None` for a resolver to unwrap.
    #[test]
    fn a_device_without_a_base_url_still_yields_one() {
        let device = Device {
            id: "abc".into(),
            name: "studio".into(),
            host: "192.168.1.24".into(),
            port: 5051,
            app: "music-player".into(),
            ..Default::default()
        };
        let config = ProviderConfig::from_device(&device, Some(5053)).unwrap();
        assert_eq!(config.url, "http://192.168.1.24:5053");
        assert_eq!(config.kind, "music-player");

        let fallback = ProviderConfig::from_device(&device, None).unwrap();
        assert_eq!(fallback.url, "http://192.168.1.24:5051");
    }

    #[test]
    fn a_configured_server_keeps_its_base_url() {
        let device = Device {
            id: "def".into(),
            name: "Navidrome".into(),
            host: "music.home.lan".into(),
            port: 4533,
            app: "subsonic".into(),
            base_url: Some("http://music.home.lan:4533/".into()),
            ..Default::default()
        };
        let config = ProviderConfig::from_device(&device, None).unwrap();
        assert_eq!(config.url, "http://music.home.lan:4533");
    }

    #[test]
    fn round_trips_through_a_device() {
        let config = ProviderConfig::new("jellyfin", "Media", "http://media.lan:8096");
        let device = config.to_device();
        assert_eq!(device.host, "media.lan");
        assert_eq!(device.port, 8096);
        assert!(device.is_source_device);
        assert!(device.is_connected);
        assert_eq!(device.base_url.as_deref(), Some("http://media.lan:8096"));
    }
}
