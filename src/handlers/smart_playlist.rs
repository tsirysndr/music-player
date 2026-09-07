//! Key handling for the TUI's smart-playlist form.
//!
//! Mirrors the fuzzy finder's overlay: while the form is up it takes every key,
//! so a stray `p` types into a field rather than pausing playback.

use crate::{
    app::App,
    event::Key,
    network::IoEvent,
    smart_playlist_form::{Field, Order},
};

pub fn handler(key: Key, app: &mut App) {
    match key {
        Key::Esc => app.smart_playlist_form.close(),
        Key::Tab => {
            let was_filter = app.smart_playlist_form.focus;
            app.smart_playlist_form.focus = app.smart_playlist_form.focus.next();
            // Moving off a field that changes the match is when the count is
            // worth a round trip — not on every keystroke.
            refresh_if_needed(app, was_filter);
        }
        Key::Down => {
            let was = app.smart_playlist_form.focus;
            app.smart_playlist_form.focus = app.smart_playlist_form.focus.next();
            refresh_if_needed(app, was);
        }
        Key::Up => {
            let was = app.smart_playlist_form.focus;
            app.smart_playlist_form.focus = app.smart_playlist_form.focus.previous();
            refresh_if_needed(app, was);
        }
        // Ctrl+O flips the sort order — a plain key would type into the field.
        Key::Ctrl('o') => {
            app.smart_playlist_form.order = app.smart_playlist_form.order.toggled();
        }
        Key::Ctrl('u') => app.smart_playlist_form.clear_field(),
        Key::Backspace => app.smart_playlist_form.backspace(),
        Key::Enter => {
            // Enter on the filter checks it; Enter anywhere else submits. That
            // way the count can be seen before committing to it.
            if app.smart_playlist_form.focus == Field::Filter {
                refresh(app);
            } else {
                submit(app);
            }
        }
        Key::Char(c) => app.smart_playlist_form.push(c),
        _ => {}
    }
}

/// Ask the daemon for a match count, if the field just left affects it.
fn refresh_if_needed(app: &mut App, left: Field) {
    if matches!(left, Field::Filter | Field::SortBy | Field::Limit) {
        refresh(app);
    }
}

fn refresh(app: &mut App) {
    app.smart_playlist_form.previewing = true;
    app.dispatch(IoEvent::PreviewSmartPlaylist {
        filter: app.smart_playlist_form.filter.clone(),
        sort_by: app.smart_playlist_form.sort_by.clone(),
        limit: app.smart_playlist_form.limit_value(),
    });
}

fn submit(app: &mut App) {
    if !app.smart_playlist_form.can_submit() {
        return;
    }
    app.smart_playlist_form.submitting = true;
    app.dispatch(IoEvent::CreateSmartPlaylist {
        name: app.smart_playlist_form.name.trim().to_string(),
        filter: app.smart_playlist_form.filter.clone(),
        sort_by: app.smart_playlist_form.sort_by.clone(),
        sort_order: match app.smart_playlist_form.order {
            Order::Asc => "asc".to_string(),
            Order::Desc => "desc".to_string(),
        },
        limit: app.smart_playlist_form.limit_value(),
    });
}
