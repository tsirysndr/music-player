use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::Style,
};

use crate::{app::App, user_config::Theme};

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
    let width = width.saturating_sub(padding);
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

pub fn get_track_progress_percentage(song_progress_ms: u128, track_duration_ms: u32) -> u16 {
    if track_duration_ms == 0 {
        return 0;
    }
    let min_perc = 0_f64;
    let track_progress = std::cmp::min(song_progress_ms, track_duration_ms.into());
    let track_perc = (track_progress as f64 / f64::from(track_duration_ms)) * 100_f64;
    (min_perc.max(track_perc) as u16).min(100)
}

pub fn display_track_progress(progress: u128, track_duration: u32) -> String {
    let duration = millis_to_minutes(u128::from(track_duration));
    let progress_display = millis_to_minutes(progress);
    let remaining = millis_to_minutes(u128::from(track_duration).saturating_sub(progress));

    format!("{}/{} (-{})", progress_display, duration, remaining,)
}

/// Returns a rect centered inside `area`, occupying `percent_x` x `percent_y`
/// of it.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1]);

    horizontal[1]
}

/// Returns a rect of at most `width` x `height`, centered inside `area`.
pub fn centered_rect_absolute(width: u16, height: u16, area: Rect) -> Rect {
    let width = width.min(area.width);
    let height = height.min(area.height);
    Rect {
        x: area.x + (area.width - width) / 2,
        y: area.y + (area.height - height) / 2,
        width,
        height,
    }
}
