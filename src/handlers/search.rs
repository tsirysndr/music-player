use crate::{
    app::{ActiveBlock, App, RouteId, SearchScope},
    event::Key,
    network::IoEvent,
};

/// Key handler for the fzf-style fuzzy finder overlay.
pub fn handler(key: Key, app: &mut App) {
    match key {
        Key::Esc => {
            app.close_search();
        }
        Key::Tab => {
            app.search.scope = app.search.scope.next();
            app.search.selected_index = 0;
            app.refresh_search_results();
        }
        Key::Down | Key::Ctrl('j') | Key::Ctrl('n') => {
            if !app.search.results.is_empty() {
                app.search.selected_index =
                    (app.search.selected_index + 1) % app.search.results.len();
            }
        }
        Key::Up | Key::Ctrl('k') | Key::Ctrl('p') => {
            if !app.search.results.is_empty() {
                app.search.selected_index = app
                    .search
                    .selected_index
                    .checked_sub(1)
                    .unwrap_or(app.search.results.len() - 1);
            }
        }
        Key::Enter => {
            submit(app);
        }
        Key::Backspace => {
            app.search.query.pop();
            app.search.selected_index = 0;
            app.refresh_search_results();
        }
        Key::Ctrl('u') => {
            app.search.query.clear();
            app.search.selected_index = 0;
            app.refresh_search_results();
        }
        Key::Char(c) => {
            app.search.query.push(c);
            app.search.selected_index = 0;
            app.refresh_search_results();
        }
        _ => {}
    }
}

fn submit(app: &mut App) {
    let selected = app
        .search
        .results
        .get(app.search.selected_index)
        .map(|result| result.id.clone());

    let id = match selected {
        Some(id) => id,
        None => return,
    };

    match app.search.scope {
        SearchScope::Tracks => {
            // Same flow as pressing Enter in a track table: add to the
            // tracklist (which starts playback) and refresh the context.
            app.dispatch(IoEvent::PlayTrack(id));
            app.dispatch(IoEvent::GetCurrentPlayback);
        }
        SearchScope::Albums => {
            app.dispatch(IoEvent::GetAlbum(id));
            app.push_navigation_stack(RouteId::AlbumTracks, ActiveBlock::AlbumTracks);
        }
        SearchScope::Artists => {
            app.dispatch(IoEvent::GetArtist(id));
            app.push_navigation_stack(RouteId::Artist, ActiveBlock::ArtistBlock);
        }
    }
    app.close_search();
}
