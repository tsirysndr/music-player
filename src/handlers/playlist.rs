use crate::{
    app::{ActiveBlock, App, RouteId},
    event::Key,
    network::IoEvent,
};

use super::common_key_events;

pub fn handler(key: Key, app: &mut App) {
    match key {
        k if common_key_events::right_event(k) => common_key_events::handle_right_event(app),
        k if common_key_events::down_event(k) => {
            let next_index = common_key_events::on_down_press_handler(
                &app.playlists,
                app.selected_playlist_index,
            );
            app.selected_playlist_index = Some(next_index);
        }
        k if common_key_events::up_event(k) => {
            let next_index =
                common_key_events::on_up_press_handler(&app.playlists, app.selected_playlist_index);
            app.selected_playlist_index = Some(next_index);
        }
        k if common_key_events::high_event(k) => {
            app.selected_playlist_index = Some(common_key_events::on_high_press_handler());
        }
        k if common_key_events::middle_event(k) => {
            if !app.playlists.is_empty() {
                app.selected_playlist_index =
                    Some(common_key_events::on_middle_press_handler(&app.playlists));
            }
        }
        k if common_key_events::low_event(k) => {
            if !app.playlists.is_empty() {
                app.selected_playlist_index =
                    Some(common_key_events::on_low_press_handler(&app.playlists));
            }
        }
        Key::Enter => {
            let playlist_id = app
                .selected_playlist_index
                .and_then(|index| app.playlists.get(index))
                .map(|playlist| playlist.id.clone());
            if let Some(id) = playlist_id {
                app.dispatch(IoEvent::PlayPlaylist(id));
                app.dispatch(IoEvent::GetPlayQueue);
                app.dispatch(IoEvent::GetCurrentPlayback);
                app.push_navigation_stack(RouteId::PlayQueue, ActiveBlock::PlayQueue);
            }
        }
        _ => (),
    }
}
