use tui::style::Color;

use crate::event::Key;

#[derive(Copy, Clone, Debug)]
pub struct Theme {
    pub active: Color,
    pub banner: Color,
    pub error_border: Color,
    pub error_text: Color,
    pub hint: Color,
    pub hovered: Color,
    pub inactive: Color,
    pub playbar_background: Color,
    pub playbar_progress: Color,
    pub playbar_progress_text: Color,
    pub playbar_text: Color,
    pub selected: Color,
    pub text: Color,
    pub header: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            active: Color::Cyan,
            banner: Color::LightCyan,
            error_border: Color::Red,
            error_text: Color::LightRed,
            hint: Color::Yellow,
            hovered: Color::Magenta,
            inactive: Color::Gray,
            playbar_background: Color::Black,
            playbar_progress: Color::LightCyan,
            playbar_progress_text: Color::LightCyan,
            playbar_text: Color::Reset,
            selected: Color::LightCyan,
            text: Color::Reset,
            header: Color::Reset,
        }
    }
}

#[derive(Clone)]
pub struct KeyBindings {
    pub back: Key,
    pub next_page: Key,
    pub previous_page: Key,
    pub jump_to_start: Key,
    pub jump_to_end: Key,
    pub jump_to_album: Key,
    pub jump_to_artist_album: Key,
    pub jump_to_context: Key,
    pub decrease_volume: Key,
    pub increase_volume: Key,
    pub toggle_playback: Key,
    pub seek_backwards: Key,
    pub seek_forwards: Key,
    pub next_track: Key,
    pub previous_track: Key,
    pub shuffle: Key,
    pub repeat: Key,
    pub search: Key,
    pub submit: Key,
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
                next_page: Key::Ctrl('d'),
                previous_page: Key::Ctrl('u'),
                jump_to_start: Key::Ctrl('a'),
                jump_to_end: Key::Ctrl('e'),
                jump_to_album: Key::Char('a'),
                jump_to_artist_album: Key::Char('A'),
                jump_to_context: Key::Char('o'),
                decrease_volume: Key::Char('-'),
                increase_volume: Key::Char('+'),
                toggle_playback: Key::Char(' '),
                seek_backwards: Key::Char('<'),
                seek_forwards: Key::Char('>'),
                next_track: Key::Char('n'),
                previous_track: Key::Char('p'),
                shuffle: Key::Ctrl('s'),
                repeat: Key::Ctrl('r'),
                search: Key::Char('/'),
                submit: Key::Enter,
                add_item_to_queue: Key::Char('z'),
            },
            behavior: BehaviorConfig {
                enable_text_emphasis: true,
            },
        }
    }
}
