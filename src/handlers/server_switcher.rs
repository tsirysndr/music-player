//! Key handling for the TUI's server switcher.
//!
//! Mirrors the fuzzy finder: while the overlay is up it takes every key, so a
//! stray `p` types into the filter rather than pausing playback.

use crate::{app::App, event::Key, network::IoEvent, server_switcher::AddField};

pub fn handler(key: Key, app: &mut App) {
    if app.switcher.form.active {
        form_handler(key, app);
        return;
    }

    match key {
        Key::Esc => app.switcher.close(),
        Key::Down | Key::Ctrl('j') => app.switcher.move_down(),
        Key::Up | Key::Ctrl('k') => app.switcher.move_up(),
        Key::Ctrl('u') => {
            app.switcher.query.clear();
            app.switcher.refresh();
        }
        // Ctrl+n adds one: a plain `n` would type into the filter.
        Key::Ctrl('n') => {
            app.switcher.form = Default::default();
            app.switcher.form.active = true;
        }
        // Ctrl+d forgets the highlighted server. The local row is not a saved
        // one, so there is nothing to forget there.
        Key::Ctrl('d') => {
            if let Some(entry) = app.switcher.selected() {
                if !entry.is_local() {
                    let id = entry.id.clone();
                    app.dispatch(IoEvent::DeleteServer(id));
                }
            }
        }
        Key::Enter => activate(app),
        Key::Backspace => {
            app.switcher.query.pop();
            app.switcher.refresh();
        }
        Key::Char(c) => {
            app.switcher.query.push(c);
            app.switcher.refresh();
        }
        _ => {}
    }
}

/// Connect to the highlighted server, or go back to the local library.
fn activate(app: &mut App) {
    let Some(entry) = app.switcher.selected().cloned() else {
        return;
    };
    app.switcher.close();
    if entry.is_local() {
        app.dispatch(IoEvent::DisconnectServer);
    } else {
        app.dispatch(IoEvent::ConnectServer(entry.id));
    }
}

fn form_handler(key: Key, app: &mut App) {
    match key {
        // Escape backs out to the list rather than closing the whole overlay.
        Key::Esc => app.switcher.form.active = false,
        Key::Tab | Key::Down => app.switcher.form.focus = app.switcher.form.focus.next(),
        Key::Up => app.switcher.form.focus = app.switcher.form.focus.previous(),
        // The kind is a choice; left/right cycle it.
        Key::Right if app.switcher.form.focus == AddField::Kind => {
            let count = app.switcher.kinds.len().max(1);
            app.switcher.form.kind = (app.switcher.form.kind + 1) % count;
        }
        Key::Left if app.switcher.form.focus == AddField::Kind => {
            let count = app.switcher.kinds.len().max(1);
            app.switcher.form.kind = (app.switcher.form.kind + count - 1) % count;
        }
        Key::Ctrl('u') => app.switcher.form.clear_field(),
        Key::Backspace => app.switcher.form.backspace(),
        Key::Enter => submit(app),
        Key::Char(c) => app.switcher.form.push(c),
        _ => {}
    }
}

fn submit(app: &mut App) {
    if !app.switcher.form.can_submit() {
        app.switcher.form.error = "a server needs a url".to_string();
        return;
    }
    let kind = app
        .switcher
        .kinds
        .get(app.switcher.form.kind)
        .map(|(kind, _)| kind.clone())
        .unwrap_or_else(|| "subsonic".to_string());
    let form = app.switcher.form.clone();
    app.switcher.form.submitting = true;
    app.dispatch(IoEvent::AddServer {
        kind,
        name: form.name.trim().to_string(),
        url: form.url.trim().to_string(),
        username: form.username.clone(),
        password: form.password.clone(),
    });
}
