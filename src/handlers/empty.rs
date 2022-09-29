use super::common_key_events;
use crate::{
    app::{ActiveBlock, App},
    event::Key,
};

// When no block is actively selected, just handle regular event
pub fn handler(key: Key, app: &mut App) {
    match key {
        Key::Enter => {
            let current_hovered = app.get_current_route().hovered_block;
            app.set_current_route_state(Some(current_hovered), None);
        }
        k if common_key_events::down_event(k) => match app.get_current_route().hovered_block {
            ActiveBlock::Library => {
                app.set_current_route_state(None, Some(ActiveBlock::Playlists));
            }
            ActiveBlock::ArtistBlock
            | ActiveBlock::AlbumList
            | ActiveBlock::AlbumTracks
            | ActiveBlock::Artists
            | ActiveBlock::TrackTable => {
                app.set_current_route_state(None, Some(ActiveBlock::PlayBar));
            }
            _ => {}
        },
        k if common_key_events::up_event(k) => match app.get_current_route().hovered_block {
            ActiveBlock::Playlists => {
                app.set_current_route_state(None, Some(ActiveBlock::Library));
            }
            ActiveBlock::PlayBar => {
                app.set_current_route_state(None, Some(ActiveBlock::Playlists));
            }
            _ => {}
        },
        k if common_key_events::left_event(k) => match app.get_current_route().hovered_block {
            ActiveBlock::ArtistBlock
            | ActiveBlock::AlbumList
            | ActiveBlock::AlbumTracks
            | ActiveBlock::Artists
            | ActiveBlock::TrackTable => {
                app.set_current_route_state(None, Some(ActiveBlock::Library));
            }
            _ => {}
        },
        k if common_key_events::right_event(k) => common_key_events::handle_right_event(app),
        _ => (),
    };
}
