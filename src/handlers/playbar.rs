use crate::{
    app::{ActiveBlock, App},
    event::Key,
};

use super::common_key_events;

pub fn handler(key: Key, app: &mut App) {
    match key {
        k if common_key_events::left_event(k) => {
            app.seek_backwards();
        }
        k if common_key_events::right_event(k) => {
            app.seek_forwards();
        }
        k if common_key_events::up_event(k) => {
            app.set_current_route_state(Some(ActiveBlock::Empty), Some(ActiveBlock::Playlists));
        }
        _ => (),
    }
}
