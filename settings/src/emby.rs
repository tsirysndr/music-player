//! Shared runtime settings for registry authentication and queued source playback.

use config::Config;
use std::sync::OnceLock;
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmbyRuntimeSettings {
    pub device_id: String,
    pub follow_redirects: bool,
}

impl EmbyRuntimeSettings {
    pub fn read() -> Self {
        Self::from_config(crate::read_settings().ok().as_ref())
    }

    fn from_config(config: Option<&Config>) -> Self {
        // Share the fallback for this process: a missing/blank device setting
        // must not give registry authentication and each resolver different IDs.
        static FALLBACK_DEVICE: OnceLock<String> = OnceLock::new();
        let device_id = config
            .and_then(|config| config.get_string("device_id").ok())
            .filter(|id| !id.trim().is_empty())
            .unwrap_or_else(|| {
                FALLBACK_DEVICE
                    .get_or_init(|| Uuid::new_v4().to_string())
                    .clone()
            });
        let follow_redirects = config
            .and_then(|config| config.get_bool("emby_follow_redirects").ok())
            .unwrap_or(true);
        Self {
            device_id,
            follow_redirects,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Exercise read_settings' file/default merge without touching user files
    // or the process environment. In this path the key exists after merging,
    // so from_config's absent/blank fallback cannot make the ID stable.
    fn config_with_device_default(toml: &str) -> Config {
        Config::builder()
            .add_source(config::File::from_str(toml, config::FileFormat::Toml))
            .set_default("device_id", crate::default_device_id())
            .unwrap()
            .build()
            .unwrap()
    }

    #[test]
    fn existing_config_without_device_id_shares_the_process_default() {
        let first = config_with_device_default("emby_follow_redirects = false");
        let second = config_with_device_default("emby_follow_redirects = false");
        let first_id = first.get_string("device_id").unwrap();
        assert_eq!(first_id, second.get_string("device_id").unwrap());
        assert_eq!(first_id.len(), 32);
        assert!(first_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)));

        let factory = EmbyRuntimeSettings::from_config(Some(&first));
        let resolver = EmbyRuntimeSettings::from_config(Some(&second));
        assert_eq!(factory.device_id, first_id);
        assert_eq!(factory, resolver);
        assert!(!factory.follow_redirects);
    }

    #[test]
    fn configured_device_id_overrides_the_process_default() {
        let config = config_with_device_default("device_id = 'configured-device-1'");
        assert_eq!(
            config.get_string("device_id").unwrap(),
            "configured-device-1"
        );
        let settings = EmbyRuntimeSettings::from_config(Some(&config));
        assert_eq!(settings.device_id, "configured-device-1");
        assert!(settings.follow_redirects);
    }

    #[test]
    fn explicit_redirect_opt_out_survives_partial_settings() {
        // Read only these keys, without requiring a complete Settings document.
        let config = Config::builder()
            .set_override("device_id", "configured-device-1")
            .unwrap()
            .set_override("emby_follow_redirects", false)
            .unwrap()
            .build()
            .unwrap();
        let settings = EmbyRuntimeSettings::from_config(Some(&config));
        assert_eq!(settings.device_id, "configured-device-1");
        assert!(!settings.follow_redirects);
    }

    #[test]
    fn absent_or_blank_device_uses_one_uuid_without_losing_redirect_opt_out() {
        let fallback = EmbyRuntimeSettings::from_config(None);
        assert!(fallback.follow_redirects);
        assert!(Uuid::parse_str(&fallback.device_id).is_ok());
        for device_id in [None, Some(""), Some("  ")] {
            let mut builder = Config::builder()
                .set_override("emby_follow_redirects", false)
                .unwrap();
            if let Some(device_id) = device_id {
                builder = builder.set_override("device_id", device_id).unwrap();
            }
            let config = builder.build().unwrap();
            let settings = EmbyRuntimeSettings::from_config(Some(&config));
            assert_eq!(settings.device_id, fallback.device_id);
            assert!(!settings.follow_redirects);
        }
    }
}
