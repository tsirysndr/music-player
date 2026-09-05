use crate::{
    app::{App, PagedCollection},
    event::Key,
    network::IoEvent,
};

use super::common_key_events;

pub fn handler(key: Key, app: &mut App) {
    match key {
        k if common_key_events::left_event(k) => common_key_events::handle_left_event(app),
        k if common_key_events::down_event(k) => {
            let next_index = common_key_events::on_down_press_handler(
                &app.track_table.tracks,
                Some(app.track_table.selected_index),
            );
            app.track_table.selected_index = next_index;
        }
        k if common_key_events::up_event(k) => {
            let next_index = common_key_events::on_up_press_handler(
                &app.track_table.tracks,
                Some(app.track_table.selected_index),
            );
            app.track_table.selected_index = next_index;
        }
        k if common_key_events::high_event(k) => {
            app.track_table.selected_index = common_key_events::on_high_press_handler();
        }
        k if common_key_events::middle_event(k) => {
            if !app.track_table.tracks.is_empty() {
                app.track_table.selected_index =
                    common_key_events::on_middle_press_handler(&app.track_table.tracks);
            }
        }
        k if common_key_events::low_event(k) => {
            if !app.track_table.tracks.is_empty() {
                app.track_table.selected_index =
                    common_key_events::on_low_press_handler(&app.track_table.tracks);
            }
        }
        _ if key == app.user_config.keys.add_item_to_queue => {
            let track_id = app
                .track_table
                .tracks
                .get(app.track_table.selected_index)
                .map(|track| track.id.clone());
            if let Some(id) = track_id {
                app.dispatch(IoEvent::AddItemToQueue(id));
            }
        }
        Key::Enter => {
            let track_id = app
                .track_table
                .tracks
                .get(app.track_table.selected_index)
                .map(|track| track.id.clone());
            if let Some(id) = track_id {
                app.dispatch(IoEvent::PlayTrack(id));
                app.dispatch(IoEvent::GetCurrentPlayback);
            }
        }
        _ => (),
    }
    // Fetch the next page when the selection gets close to the end of the
    // loaded items (no-op for fully loaded or non-paged track lists).
    app.maybe_load_more(PagedCollection::Tracks);
}
