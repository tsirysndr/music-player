use crate::{
    app::{ActiveBlock, App},
    event::Key,
    network::IoEvent,
};

pub mod album_tracks;
pub mod albums;
pub mod artist_tracks;
pub mod artists;
pub mod common_key_events;
pub mod empty;
pub mod input;
pub mod library;
pub mod play_queue;
pub mod playbar;
pub mod playlist;
pub mod tracks;

pub fn handle_app(key: Key, app: &mut App) {
    match key {
        Key::Esc => {
            handle_escape(app);
        }
        _ if key == app.user_config.keys.jump_to_album => {
            handle_jump_to_album(app);
        }

        _ if key == app.user_config.keys.jump_to_artist_album => {
            handle_jump_to_artist_album(app);
        }
        _ if key == app.user_config.keys.jump_to_context => {
            handle_jump_to_context(app);
        }
        _ if key == app.user_config.keys.decrease_volume => {
            app.decrease_volume();
        }
        _ if key == app.user_config.keys.increase_volume => {
            app.increase_volume();
        }
        // Press space to toggle playback
        _ if key == app.user_config.keys.toggle_playback => {
            app.toggle_playback();
        }
        _ if key == app.user_config.keys.seek_backwards => {
            app.seek_backwards();
        }
        _ if key == app.user_config.keys.seek_forwards => {
            app.seek_forwards();
        }
        _ if key == app.user_config.keys.next_track => {
            app.dispatch(IoEvent::NextTrack);
        }
        _ if key == app.user_config.keys.previous_track => {
            app.dispatch(IoEvent::PreviousTrack);
        }
        _ if key == app.user_config.keys.shuffle => {
            app.shuffle();
        }
        _ if key == app.user_config.keys.repeat => {
            app.repeat();
        }
        _ if key == app.user_config.keys.search => {
            app.set_current_route_state(Some(ActiveBlock::Input), Some(ActiveBlock::Input));
        }

        _ => handle_block_events(key, app),
    }
}

fn handle_block_events(key: Key, app: &mut App) {
    let current_route = app.get_current_route();
    match current_route.active_block {
        ActiveBlock::Input => {
            input::handler(key, app);
        }
        ActiveBlock::PlayBar => {
            playbar::handler(key, app);
        }
        ActiveBlock::AlbumTracks => {
            tracks::handler(key, app);
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
        ActiveBlock::SearchResultBlock => todo!(),
        ActiveBlock::PlayQueue => {
            play_queue::handler(key, app);
        }
        ActiveBlock::Empty => empty::handler(key, app),
    }
}

fn handle_escape(app: &mut App) {
    match app.get_current_route().active_block {
        ActiveBlock::Input => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Library));
        }
        /*
        ActiveBlock::SearchResultBlock => {
             app.search_results.selected_block = SearchResultBlock::Empty;
         }
         ActiveBlock::ArtistBlock => {
             if let Some(artist) = &mut app.artist {
                 artist.artist_selected_block = ArtistBlock::Empty;
             }
         }
        */
        _ => {
            app.set_current_route_state(Some(ActiveBlock::Empty), None);
        }
    }
}

fn handle_jump_to_album(app: &mut App) {}

fn handle_jump_to_artist(app: &mut App) {}

fn handle_jump_to_artist_album(app: &mut App) {}

fn handle_jump_to_context(app: &mut App) {}
