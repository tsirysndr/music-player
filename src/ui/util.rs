use tui::style::Style;

use crate::{app::App, user_config::Theme};

pub const BASIC_VIEW_HEIGHT: u16 = 6;
pub const SMALL_TERMINAL_WIDTH: u16 = 150;
pub const SMALL_TERMINAL_HEIGHT: u16 = 45;

pub fn get_main_layout_margin(app: &App) -> u16 {
    if app.size.height > SMALL_TERMINAL_HEIGHT {
        1
    } else {
        0
    }
}

pub fn get_color((is_active, is_hovered): (bool, bool), theme: Theme) -> Style {
    match (is_active, is_hovered) {
        (true, _) => Style::default().fg(theme.selected),
        (false, true) => Style::default().fg(theme.hovered),
        _ => Style::default().fg(theme.inactive),
    }
}

// `percentage` param needs to be between 0 and 1
pub fn get_percentage_width(width: u16, percentage: f32) -> u16 {
    let padding = 3;
    let width = width - padding;
    (f32::from(width) * percentage) as u16
}

pub fn millis_to_minutes(millis: u128) -> String {
    let minutes = millis / 60000;
    let seconds = (millis % 60000) / 1000;
    let seconds_display = if seconds < 10 {
        format!("0{}", seconds)
    } else {
        format!("{}", seconds)
    };

    if seconds == 60 {
        format!("{}:00", minutes + 1)
    } else {
        format!("{}:{}", minutes, seconds_display)
    }
}
