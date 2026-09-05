use ratatui::style::Color;

use crate::event::Key;

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
            active: Color::Cyan,
            hovered: Color::Magenta,
            inactive: Color::Gray,
            playbar_background: Color::Black,
            playbar_progress: Color::LightCyan,
            playbar_progress_text: Color::LightCyan,
            playbar_text: Color::Reset,
            selected: Color::LightCyan,
            text: Color::Reset,
            header: Color::Reset,
            hint: Color::Yellow,
            statusline_normal: Color::Cyan,
            statusline_search: Color::Yellow,
            statusline_bg: Color::DarkGray,
            statusline_fg: Color::White,
        }
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
            theme: Default::default(),
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
            },
            behavior: BehaviorConfig {
                enable_text_emphasis: true,
            },
        }
    }
}
