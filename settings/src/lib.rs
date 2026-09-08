#[cfg(test)]
mod tests;

use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use config::{Config, ConfigError};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_RADIO_BROWSER_URL: &str = "https://de1.api.radio-browser.info";
pub const DEFAULT_TUNEIN_URL: &str = "https://opml.radiotime.com";
/// atradio.fm's media proxy. Stations whose playlist or stream the origin will
/// not serve us directly are played through it — see `media_proxy_url`.
pub const DEFAULT_MEDIA_PROXY_URL: &str = "https://media.atradio.fm";

/// Audio/DSP settings persisted in the `[audio]` table of settings.toml and
/// applied to the playback engine at boot. Integer enums follow the Rockbox
/// firmware conventions used by the desktop UI:
/// - `replaygain_mode`: 0 track, 1 album, 2 track (shuffle), 3 off
/// - `crossfade`: 0 off, 1 auto track change, 2 manual track change,
///   3 shuffle, 4 shuffle or manual skip, 5 always
/// - `fade_out_mixmode`: 0 crossfade, 2 mix
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct AudioSettings {
    pub eq_enabled: bool,
    /// EQ pre-gain (headroom) in dB, 0..=24.
    pub eq_precut: f32,
    /// Per-band gains in dB (-24..=24), one per EQ band
    /// (32 Hz … 16 kHz, octave-spaced).
    pub eq_band_gains: Vec<f32>,
    /// Bass shelf gain in dB, -24..=24.
    pub bass: i32,
    /// Treble shelf gain in dB, -24..=24.
    pub treble: i32,
    /// Stereo balance, -100 (full left)..=100 (full right).
    pub balance: i32,
    pub replaygain_mode: i32,
    /// ReplayGain pre-amp in dB, -12.0..=12.0.
    pub replaygain_preamp: f32,
    pub replaygain_noclip: bool,
    pub crossfade: i32,
    /// Crossfade fade-in delay in seconds, 0..=7.
    pub fade_in_delay: u64,
    /// Crossfade fade-in duration in seconds, 0..=15.
    pub fade_in_duration: u64,
    /// Crossfade fade-out delay in seconds, 0..=7.
    pub fade_out_delay: u64,
    /// Crossfade fade-out duration in seconds, 0..=15.
    pub fade_out_duration: u64,
    pub fade_out_mixmode: i32,
    pub dithering: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            eq_enabled: false,
            eq_precut: 0.0,
            eq_band_gains: vec![0.0; 10],
            bass: 0,
            treble: 0,
            balance: 0,
            replaygain_mode: 3, // off
            replaygain_preamp: 0.0,
            replaygain_noclip: false,
            crossfade: 0, // off
            fade_in_delay: 0,
            fade_in_duration: 2,
            fade_out_delay: 0,
            fade_out_duration: 2,
            fade_out_mixmode: 0,
            dithering: false,
        }
    }
}

/// Optional Typesense search backend. When this `[typesense]` table is
/// present in settings.toml (url + api key), library search runs against
/// Typesense instead of the built-in SQLite FTS5 index; the collections are
/// re-synced after every library scan.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct TypesenseSettings {
    /// Base URL, e.g. "http://localhost:8108".
    pub url: String,
    /// The `x-typesense-api-key` credential.
    pub api_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Settings {
    pub database_url: String,
    pub port: u16,
    pub ws_port: u16,
    pub addons: Option<Vec<String>>,
    pub music_directory: String,
    pub host: String,
    pub device_name: String,
    pub device_id: String,
    pub http_port: u16,
    /// Where decoded audio is sent: "cpal" (system audio device, default),
    /// "stdout", "fifo:/path/to/pipe", "unix:/path/to/socket" or "tcp:host:port".
    pub audio_output: String,
    /// How often (in minutes) the music directory is rescanned in the
    /// background. 0 disables periodic refresh.
    pub library_refresh_interval: u64,
    /// Radio Browser API base URL, without a trailing path.
    pub radio_browser_url: String,
    /// TuneIn OPML API base URL, without a trailing path.
    pub tunein_url: String,
    /// Where to look for WebAssembly extensions, in order. Each entry is a
    /// directory holding one subdirectory per extension. Empty falls back to
    /// `<app dir>/extensions`; listing several lets a user keep their own
    /// extensions apart from ones installed by a package manager.
    ///
    /// A `~` prefix and `$VAR` are expanded.
    #[serde(default)]
    pub extension_paths: Vec<String>,
    /// Base URL of a CORS/relay media proxy exposing `/api/stream?url=`.
    /// Used as the fallback for stations that refuse a direct connection —
    /// a bot wall in front of a station's `.pls` is common, and the proxy is
    /// the only way past it. Empty disables the fallback: an unreachable
    /// station then simply fails to play.
    pub media_proxy_url: String,
    /// Base URL of a Subsonic-compatible server (Navidrome, Airsonic, gonic, ...).
    /// Empty or absent means the Subsonic integration is disabled.
    pub subsonic_url: Option<String>,
    pub subsonic_username: Option<String>,
    pub subsonic_password: Option<String>,
    /// Base URL of a Jellyfin server. Empty or absent means the Jellyfin
    /// integration is disabled.
    pub jellyfin_url: Option<String>,
    pub jellyfin_username: Option<String>,
    pub jellyfin_password: Option<String>,
    /// Rocksky auto-scrobbling (needs `rocksky login`).
    #[serde(default = "default_true")]
    pub scrobble: bool,
    /// atproto integration: import radio bookmarks and liked songs from the
    /// user's repo, write bookmarks and the now-listening status back, and
    /// follow the repo on Jetstream. Off means nothing atproto runs at all,
    /// whether or not credentials happen to be present.
    #[serde(default = "default_true")]
    pub atproto: bool,
    /// Download the repo CAR archive on every start, instead of only when the
    /// last download has aged out. Same effect as `--force-car-sync`.
    #[serde(default)]
    pub atproto_force_car_sync: bool,
    /// How old the last repo CAR download may be, in hours, before the repo is
    /// pulled again. 0 means every start.
    #[serde(default = "default_car_max_age_hours")]
    pub atproto_car_max_age_hours: u64,
    /// Register as a Rocksky remote-player device (needs `rocksky login`),
    /// so the daemon shows up in the web/desktop miniplayer device picker.
    #[serde(default = "default_true")]
    pub remote_player: bool,
    // Keep tables last: toml requires them after plain values when
    // serializing.
    /// Optional Typesense search backend; absent = SQLite FTS5.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typesense: Option<TypesenseSettings>,
    #[serde(default)]
    pub audio: AudioSettings,
}

/// The `[typesense]` section of settings.toml, if configured with a
/// non-empty url. Reads the file directly so callers don't need a full
/// `Settings` deserialization round-trip.
pub fn read_typesense_settings() -> Option<TypesenseSettings> {
    let config = read_settings().ok()?;
    let settings = config.try_deserialize::<Settings>().ok()?;
    settings.typesense.filter(|t| !t.url.trim().is_empty())
}

fn default_true() -> bool {
    true
}

/// A day: long enough that a restart does not re-download a whole repo, short
/// enough that a repo edited elsewhere is picked up without asking.
fn default_car_max_age_hours() -> u64 {
    24
}

pub fn read_settings() -> Result<Config, ConfigError> {
    let path = match env::consts::OS {
        "android" => "/storage/emulated/0/Android/data/com.tsirysndr.songbird/files".to_owned(),
        _ => {
            let config_dir = dirs::config_dir().unwrap();
            format!("{}/music-player", config_dir.to_str().unwrap())
        }
    };

    let music_directory = match env::consts::OS {
        "android" => "/storage/emulated/0/Music".to_owned(),
        _ => {
            let mut tmp = PathBuf::new();
            tmp.push("/tmp");
            let music_dir = dirs::audio_dir().unwrap_or(tmp);
            music_dir.to_str().unwrap().to_owned()
        }
    };

    let covers_path = format!("{}/covers", path);

    fs::create_dir_all(&covers_path).unwrap();

    let device_id = format!("{:x}", md5::compute(Uuid::new_v4().to_string()));

    let mut tmp = PathBuf::new();
    tmp.push("/tmp");

    let default_settings = Settings {
        database_url: format!("sqlite:{}/music-player.sqlite3", path),
        port: 5051,
        ws_port: 5052,
        addons: Some(vec![
            "local".to_string(),
            "chromecast".to_string(),
            "dlna".to_string(),
            "subsonic".to_string(),
            "jellyfin".to_string(),
        ]),
        music_directory,
        host: "0.0.0.0".to_string(),
        device_name: "Music Player".to_string(),
        device_id,
        http_port: 5053,
        audio_output: "cpal".to_string(),
        library_refresh_interval: 30,
        radio_browser_url: DEFAULT_RADIO_BROWSER_URL.to_string(),
        tunein_url: DEFAULT_TUNEIN_URL.to_string(),
        media_proxy_url: DEFAULT_MEDIA_PROXY_URL.to_string(),
        // Empty means "just the default under the app directory".
        extension_paths: vec![],
        subsonic_url: Some("".to_string()),
        subsonic_username: Some("".to_string()),
        subsonic_password: Some("".to_string()),
        jellyfin_url: Some("".to_string()),
        jellyfin_username: Some("".to_string()),
        jellyfin_password: Some("".to_string()),
        scrobble: true,
        atproto: true,
        atproto_force_car_sync: false,
        atproto_car_max_age_hours: default_car_max_age_hours(),
        remote_player: true,
        typesense: None,
        audio: AudioSettings::default(),
    };

    let settings_path = format!("{}/settings.toml", path);

    if !Path::new(&settings_path).exists() {
        let mut file = File::create(format!("{}/settings.toml", path)).unwrap();
        file.write_all(
            toml::to_string_pretty(&default_settings)
                .unwrap()
                .as_bytes(),
        )
        .unwrap();
    }

    Config::builder()
        .add_source(config::File::with_name(&settings_path))
        .add_source(config::Environment::with_prefix("MUSIC_PLAYER"))
        .set_default("database_url", default_settings.database_url)?
        .set_default("port", default_settings.port)?
        .set_default("addons", default_settings.addons)?
        .set_default("ws_port", default_settings.ws_port)?
        .set_default("music_directory", default_settings.music_directory)?
        .set_default("host", default_settings.host)?
        .set_default("device_name", default_settings.device_name)?
        .set_default("device_id", default_settings.device_id)?
        .set_default("http_port", default_settings.http_port)?
        .set_default("audio_output", default_settings.audio_output)?
        .set_default(
            "library_refresh_interval",
            default_settings.library_refresh_interval,
        )?
        .set_default("radio_browser_url", default_settings.radio_browser_url)?
        .set_default("tunein_url", default_settings.tunein_url)?
        .set_default("media_proxy_url", default_settings.media_proxy_url)?
        .set_default("extension_paths", default_settings.extension_paths)?
        .set_default("subsonic_url", "")?
        .set_default("subsonic_username", "")?
        .set_default("subsonic_password", "")?
        .set_default("jellyfin_url", "")?
        .set_default("jellyfin_username", "")?
        .set_default("jellyfin_password", "")?
        .set_default("scrobble", true)?
        .set_default("atproto", true)?
        .set_default("atproto_force_car_sync", false)?
        .set_default(
            "atproto_car_max_age_hours",
            default_car_max_age_hours() as i64,
        )?
        .set_default("remote_player", true)?
        .build()
}

/// Persist the `[audio]` table of settings.toml, leaving every other key
/// untouched (the file is edited in place, not regenerated, so manual
/// edits and comments outside `[audio]` survive as much as toml allows).
pub fn save_audio_settings(audio: &AudioSettings) -> std::io::Result<()> {
    let path = match env::consts::OS {
        "android" => "/storage/emulated/0/Android/data/com.tsirysndr.songbird/files".to_owned(),
        _ => {
            let config_dir = dirs::config_dir().unwrap();
            format!("{}/music-player", config_dir.to_str().unwrap())
        }
    };
    let settings_path = format!("{}/settings.toml", path);
    let contents = fs::read_to_string(&settings_path).unwrap_or_default();
    let mut doc: toml::Value =
        toml::from_str(&contents).unwrap_or(toml::Value::Table(Default::default()));
    let audio_value = toml::Value::try_from(audio)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(table) = doc.as_table_mut() {
        table.insert("audio".to_string(), audio_value);
    }
    let mut file = File::create(&settings_path)?;
    file.write_all(
        toml::to_string_pretty(&doc)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
            .as_bytes(),
    )
}

pub fn get_application_directory() -> String {
    let config_dir = match env::consts::OS {
        "android" => "/storage/emulated/0/Android/data/com.tsirysndr.songbird/files".to_owned(),
        _ => {
            let config_dir = dirs::config_dir().unwrap();
            format!("{}/music-player", config_dir.to_str().unwrap())
        }
    };
    let path = env::var("MUSIC_PLAYER_APPLICATION_DIRECTORY").unwrap_or(config_dir);
    let albums = format!("{}/albums", path);
    let artists = format!("{}/artists", path);
    let playlists = format!("{}/playlists", path);
    let tracks = format!("{}/tracks", path);
    let covers = format!("{}/covers", path);
    let cache = format!("{}/cache", path);
    fs::create_dir_all(&albums).unwrap();
    fs::create_dir_all(&artists).unwrap();
    fs::create_dir_all(&playlists).unwrap();
    fs::create_dir_all(&tracks).unwrap();
    fs::create_dir_all(&covers).unwrap();
    fs::create_dir_all(&cache).unwrap();

    path
}
