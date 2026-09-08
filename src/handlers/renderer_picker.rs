//! Key handling for the TUI's "play to" picker.
//!
//! Mirrors the server switcher: while the overlay is up it takes every key, so
//! a stray `p` types into the filter rather than skipping a track.

use crate::{app::App, event::Key, network::IoEvent};

pub fn handler(key: Key, app: &mut App) {
    match key {
        Key::Esc => app.renderers.close(),
        Key::Down | Key::Ctrl('j') => app.renderers.move_down(),
        Key::Up | Key::Ctrl('k') => app.renderers.move_up(),
        Key::Ctrl('u') => {
            app.renderers.query.clear();
            app.renderers.refresh();
        }
        Key::Enter => activate(app),
        Key::Backspace => {
            app.renderers.query.pop();
            app.renderers.refresh();
        }
        Key::Char(c) => {
            app.renderers.query.push(c);
            app.renderers.refresh();
        }
        _ => {}
    }
}

/// Hand playback to the highlighted renderer, or take it back.
fn activate(app: &mut App) {
    let Some(entry) = app.renderers.selected().cloned() else {
        return;
    };
    app.renderers.close();
    // The local row is spelled as an empty id, which is what the daemon reads
    // as "play here" — said here rather than relied on.
    let id = if entry.is_local() {
        String::new()
    } else {
        entry.id
    };
    app.dispatch(IoEvent::ActivateRenderer {
        id,
        cast: entry.cast,
    });
}
