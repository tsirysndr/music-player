use crate::{
    app::{ActiveBlock, App, RouteId},
    event::Key,
    network::IoEvent,
};

pub mod album_tracks;
pub mod albums;
pub mod artist_tracks;
pub mod artists;
pub mod common_key_events;
pub mod empty;
pub mod library;
pub mod play_queue;
pub mod playbar;
pub mod playlist;
pub mod renderer_picker;
pub mod search;
pub mod server_switcher;
pub mod smart_playlist;
pub mod tracks;

/// Handles a key press. Returns `true` when the application should exit.
pub fn handle_app(key: Key, app: &mut App) -> bool {
    // The help dialog swallows every key until it is closed.
    if app.show_help {
        match key {
            Key::Esc | Key::Char('?') | Key::Char('q') => {
                app.show_help = false;
            }
            Key::Down | Key::Char('j') => {
                app.help_scroll = app.help_scroll.saturating_add(1);
            }
            Key::Up | Key::Char('k') => {
                app.help_scroll = app.help_scroll.saturating_sub(1);
            }
            _ => {}
        }
        return false;
    }

    // The smart-playlist form takes every key while it is up, so a stray `p`
    // types into a field rather than pausing playback.
    if app.smart_playlist_form.active {
        smart_playlist::handler(key, app);
        return false;
    }
    // The fuzzy finder owns the keyboard while it is open.
    if app.search.active {
        search::handler(key, app);
        return false;
    }
    // As does the server switcher.
    if app.switcher.active {
        server_switcher::handler(key, app);
        return false;
    }
    // And the "play to" picker.
    if app.renderers.active {
        renderer_picker::handler(key, app);
        return false;
    }

    match key {
        Key::Esc => {
            handle_escape(app);
        }
        _ if key == app.user_config.keys.help => {
            app.help_scroll = 0;
            app.show_help = true;
        }
        _ if key == app.user_config.keys.search => {
            app.open_search();
        }
        _ if key == app.user_config.keys.new_smart_playlist => {
            app.smart_playlist_form.open();
        }
        // `=` and `_` are the unshifted faces of `+` and `-` on most layouts,
        // so both reach the same place rather than only the shifted one.
        _ if key == app.user_config.keys.decrease_volume || key == Key::Char('_') => {
            app.decrease_volume();
        }
        _ if key == app.user_config.keys.increase_volume || key == Key::Char('=') => {
            app.increase_volume();
        }
        _ if key == app.user_config.keys.toggle_mute => {
            app.toggle_mute();
        }
        _ if key == app.user_config.keys.play_to => {
            app.renderers.open();
            app.renderers.loading = true;
            app.dispatch(IoEvent::LoadRenderers);
        }
        _ if key == app.user_config.keys.switch_server => {
            app.switcher.open();
            app.switcher.loading = true;
            app.dispatch(IoEvent::LoadServers);
        }
        // Press space to toggle playback
        _ if key == app.user_config.keys.toggle_playback => {
            app.toggle_playback();
            app.dispatch(IoEvent::GetCurrentPlayback);
        }
        _ if key == app.user_config.keys.seek_backwards => {
            app.seek_backwards();
        }
        _ if key == app.user_config.keys.seek_forwards => {
            app.seek_forwards();
        }
        _ if key == app.user_config.keys.next_track => {
            app.dispatch(IoEvent::NextTrack);
            app.dispatch(IoEvent::GetCurrentPlayback);
        }
        _ if key == app.user_config.keys.previous_track => {
            app.dispatch(IoEvent::PreviousTrack);
            app.dispatch(IoEvent::GetCurrentPlayback);
        }
        // Toggle the play-queue view.
        _ if key == app.user_config.keys.show_queue => {
            if app.get_current_route().id == RouteId::PlayQueue {
                app.pop_navigation_stack();
            } else {
                app.dispatch(IoEvent::GetPlayQueue);
                app.push_navigation_stack(RouteId::PlayQueue, ActiveBlock::PlayQueue);
            }
        }
        _ if key == app.user_config.keys.back => {
            // Walk back through the navigation stack; exit once it is empty.
            return app.pop_navigation_stack().is_none();
        }
        _ => handle_block_events(key, app),
    }
    false
}

fn handle_block_events(key: Key, app: &mut App) {
    let current_route = app.get_current_route();
    match current_route.active_block {
        ActiveBlock::PlayBar => {
            playbar::handler(key, app);
        }
        ActiveBlock::AlbumTracks => {
            album_tracks::handler(key, app);
        }
        ActiveBlock::AlbumList => {
            albums::handler(key, app);
        }
        ActiveBlock::ArtistBlock => {
            artist_tracks::handler(key, app);
        }
        ActiveBlock::Library => {
            library::handler(key, app);
        }
        ActiveBlock::Playlists => {
            playlist::handler(key, app);
        }
        ActiveBlock::TrackTable => {
            tracks::handler(key, app);
        }
        ActiveBlock::Artists => {
            artists::handler(key, app);
        }
        ActiveBlock::PlayQueue => {
            play_queue::handler(key, app);
        }
        ActiveBlock::Empty => empty::handler(key, app),
    }
}

fn handle_escape(app: &mut App) {
    app.set_current_route_state(Some(ActiveBlock::Empty), None);
}
