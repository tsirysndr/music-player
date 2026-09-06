use music_player_settings::read_settings;
use ratatui::style::Color;

use crate::event::Key;

// Fluorescent palette (mirrors the desktop app's Synthwave skin): neon cyan
// display, hot magenta accent, UV purple hover, chartreuse hints. Rgb values
// need a truecolor terminal — every modern emulator qualifies.
const NEON_CYAN: Color = Color::Rgb(0, 229, 255);
const NEON_MAGENTA: Color = Color::Rgb(255, 45, 149);
const NEON_PURPLE: Color = Color::Rgb(195, 85, 255);
const NEON_CHARTREUSE: Color = Color::Rgb(204, 255, 0);
// SpaceVim-style statusline base: one-dark charcoal with soft gray text
// (the mode chip keeps its bright color on top of it).
const STATUSLINE_BG: Color = Color::Rgb(44, 50, 60);
const STATUSLINE_FG: Color = Color::Rgb(171, 178, 191);

#[derive(Copy, Clone, Debug)]
pub struct Theme {
    pub active: Color,
    pub hovered: Color,
    pub inactive: Color,
    pub playbar_background: Color,
    pub playbar_progress: Color,
    pub playbar_progress_text: Color,
    pub playbar_text: Color,
    pub selected: Color,
    pub text: Color,
    pub header: Color,
    pub hint: Color,
    pub statusline_normal: Color,
    pub statusline_search: Color,
    pub statusline_bg: Color,
    pub statusline_fg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            active: NEON_CYAN,
            hovered: NEON_PURPLE,
            inactive: Color::Gray,
            playbar_background: Color::Black,
            playbar_progress: NEON_MAGENTA,
            // Time readout in default fg — legible on any terminal theme.
            playbar_progress_text: Color::Reset,
            playbar_text: Color::Reset,
            selected: NEON_MAGENTA,
            text: Color::Reset,
            header: Color::Reset,
            hint: NEON_CHARTREUSE,
            statusline_normal: NEON_CYAN,
            statusline_search: NEON_CHARTREUSE,
            statusline_bg: STATUSLINE_BG,
            statusline_fg: STATUSLINE_FG,
        }
    }
}

/// Parses a color token from settings.toml: `#rrggbb` hex, or a ratatui
/// color name (`cyan`, `light-magenta`, `dark-gray`, `reset`, …).
fn parse_color(s: &str) -> Option<Color> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
        return None;
    }
    match s.to_lowercase().replace(['-', '_'], "").as_str() {
        "reset" => Some(Color::Reset),
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "darkgray" | "darkgrey" => Some(Color::DarkGray),
        "lightred" => Some(Color::LightRed),
        "lightgreen" => Some(Color::LightGreen),
        "lightyellow" => Some(Color::LightYellow),
        "lightblue" => Some(Color::LightBlue),
        "lightmagenta" => Some(Color::LightMagenta),
        "lightcyan" => Some(Color::LightCyan),
        "white" => Some(Color::White),
        _ => None,
    }
}

impl Theme {
    /// The default fluorescent theme, overridden per key by the optional
    /// `[tui_theme]` table of settings.toml. Values are `#rrggbb` hex or
    /// named colors; unknown keys/values are ignored.
    pub fn from_settings() -> Self {
        let mut theme = Theme::default();
        let Ok(config) = read_settings() else {
            return theme;
        };
        let Ok(table) = config.get_table("tui_theme") else {
            return theme;
        };
        for (key, value) in table {
            let Ok(value) = value.into_string() else {
                continue;
            };
            let Some(color) = parse_color(&value) else {
                tracing::warn!("[tui_theme] {key}: unknown color {value:?}");
                continue;
            };
            match key.as_str() {
                "active" => theme.active = color,
                "hovered" => theme.hovered = color,
                "inactive" => theme.inactive = color,
                "playbar_background" => theme.playbar_background = color,
                "playbar_progress" => theme.playbar_progress = color,
                "playbar_progress_text" => theme.playbar_progress_text = color,
                "playbar_text" => theme.playbar_text = color,
                "selected" => theme.selected = color,
                "text" => theme.text = color,
                "header" => theme.header = color,
                "hint" => theme.hint = color,
                "statusline_normal" => theme.statusline_normal = color,
                "statusline_search" => theme.statusline_search = color,
                "statusline_bg" => theme.statusline_bg = color,
                "statusline_fg" => theme.statusline_fg = color,
                other => tracing::warn!("[tui_theme] unknown key: {other}"),
            }
        }
        theme
    }
}

#[derive(Clone)]
pub struct KeyBindings {
    pub back: Key,
    pub decrease_volume: Key,
    pub increase_volume: Key,
    pub toggle_playback: Key,
    pub seek_backwards: Key,
    pub seek_forwards: Key,
    pub next_track: Key,
    pub previous_track: Key,
    pub search: Key,
    pub help: Key,
    pub add_item_to_queue: Key,
    pub show_queue: Key,
}

#[derive(Clone)]
pub struct BehaviorConfig {
    pub enable_text_emphasis: bool,
}

#[derive(Clone)]
pub struct UserConfig {
    pub theme: Theme,
    pub keys: KeyBindings,
    pub behavior: BehaviorConfig,
}

impl UserConfig {
    pub fn new() -> UserConfig {
        UserConfig {
            theme: Theme::from_settings(),
            keys: KeyBindings {
                back: Key::Char('q'),
                decrease_volume: Key::Char('-'),
                increase_volume: Key::Char('+'),
                toggle_playback: Key::Char(' '),
                seek_backwards: Key::Char('<'),
                seek_forwards: Key::Char('>'),
                next_track: Key::Char('n'),
                previous_track: Key::Char('p'),
                search: Key::Char('/'),
                help: Key::Char('?'),
                add_item_to_queue: Key::Char('z'),
                show_queue: Key::Char('u'),
            },
            behavior: BehaviorConfig {
                enable_text_emphasis: true,
            },
        }
    }
}
