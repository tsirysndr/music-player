use crate::{
    app::{ActiveBlock, App, PagedCollection, RouteId},
    event::Key,
    network::IoEvent,
};

use super::common_key_events;

pub fn handler(key: Key, app: &mut App) {
    match key {
        k if common_key_events::left_event(k) => common_key_events::handle_left_event(app),
        k if common_key_events::down_event(k) => {
            let next_index = common_key_events::on_down_press_handler(
                &app.artist_table.artists,
                Some(app.artist_table.selected_index),
            );
            app.artist_table.selected_index = next_index;
        }
        k if common_key_events::up_event(k) => {
            let next_index = common_key_events::on_up_press_handler(
                &app.artist_table.artists,
                Some(app.artist_table.selected_index),
            );
            app.artist_table.selected_index = next_index;
        }
        k if common_key_events::high_event(k) => {
            let next_index = common_key_events::on_high_press_handler();
            app.artist_table.selected_index = next_index;
        }
        Key::Enter => {
            if let Some(artist) = app
                .artist_table
                .artists
                .get(app.artist_table.selected_index)
            {
                let id = artist.id.clone();
                app.selected_artist_name = Some(artist.name.clone());
                app.dispatch(IoEvent::GetArtist(id));
                app.push_navigation_stack(RouteId::Artist, ActiveBlock::ArtistBlock);
            }
        }

        _ => (),
    }
    // Fetch the next page when the selection gets close to the end of the
    // loaded artists.
    app.maybe_load_more(PagedCollection::Artists);
}
