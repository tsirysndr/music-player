use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

use crate::{
    app::{ActiveBlock, App, RouteId, LIBRARY_OPTIONS},
    ui::util::SMALL_TERMINAL_WIDTH,
};

use self::util::get_color;

pub mod util;

pub enum TableId {
    Album,
    AlbumList,
    Artist,
    Song,
}

pub enum ColumnId {
    None,
    Title,
}

pub struct TableHeader<'a> {
    id: TableId,
    items: Vec<TableHeaderItem<'a>>,
}

pub struct TableHeaderItem<'a> {
    id: ColumnId,
    text: &'a str,
    width: u16,
}

pub struct TableItem {
    id: String,
    format: Vec<String>,
}

pub struct AlbumUi {
    selected_index: usize,
}

pub fn draw_main_layout<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let margin = util::get_main_layout_margin(app);

    let parent_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(6),
            ]
            .as_ref(),
        )
        .margin(margin)
        .split(f.size());

    draw_input(f, app, parent_layout[0]);

    // Nested main block with potential routes
    draw_routes(f, app, parent_layout[1]);
}

pub fn draw_routes<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(layout_chunk);
    draw_user_block(f, app, chunks[0]);

    let current_route = app.get_current_route();

    match current_route.id {
        RouteId::AlbumTracks => draw_album_table(f, app, chunks[1]),
        RouteId::AlbumList => draw_album_list(f, app, chunks[1]),
        RouteId::Artist => draw_artist_albums(f, app, chunks[1]),
        RouteId::Search => draw_search_results(f, app, chunks[1]),
        RouteId::TrackTable => draw_song_table(f, app, chunks[1]),
        RouteId::Artists => draw_artist_table(f, app, chunks[1]),
        RouteId::PlayQueue => draw_play_queue(f, app, chunks[1]),
    }
}

pub fn draw_user_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(layout_chunk);

    draw_library_block(f, app, chunks[0]);
    draw_playlist_block(f, app, chunks[1]);
}

pub fn draw_input<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    // Check for the width and change the contraints accordingly
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100), Constraint::Percentage(10)].as_ref())
        .split(layout_chunk);

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::Input,
        current_route.hovered_block == ActiveBlock::Input,
    );

    let input_string: String = app.input.iter().collect();
    let lines = Text::from((&input_string).as_str());
    let input = Paragraph::new(lines).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled(
                "Search",
                get_color(highlight_state, app.user_config.theme),
            ))
            .border_style(get_color(highlight_state, app.user_config.theme)),
    );
    f.render_widget(input, chunks[0]);
}

pub fn draw_library_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::Library,
        current_route.hovered_block == ActiveBlock::Library,
    );
    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Library",
        &LIBRARY_OPTIONS,
        highlight_state,
        Some(app.library.selected_index),
    );
}

pub fn draw_playlist_block<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let playlist_items = match &app.playlists {
        Some(p) => p.items.iter().map(|item| item.name.to_owned()).collect(),
        None => vec![],
    };

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::Playlists,
        current_route.hovered_block == ActiveBlock::Playlists,
    );

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Playlists",
        &playlist_items,
        highlight_state,
        app.selected_playlist_index,
    );
}

pub fn draw_search_results<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_selectable_list<B, S>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    title: &str,
    items: &[S],
    highlight_state: (bool, bool),
    selected_index: Option<usize>,
) where
    B: Backend,
    S: std::convert::AsRef<str>,
{
    let mut state = ListState::default();
    state.select(selected_index);

    let lst_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(Span::raw(i.as_ref())))
        .collect();

    let list = List::new(lst_items)
        .block(
            Block::default()
                .title(Span::styled(
                    title,
                    get_color(highlight_state, app.user_config.theme),
                ))
                .borders(Borders::ALL)
                .border_style(get_color(highlight_state, app.user_config.theme)),
        )
        .style(Style::default().fg(app.user_config.theme.text))
        .highlight_style(
            get_color(highlight_state, app.user_config.theme).add_modifier(Modifier::BOLD),
        );
    f.render_stateful_widget(list, layout_chunk, &mut state);
}

pub fn draw_album_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_artist_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let artist_items: Vec<String> = Vec::new();

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::Artists,
        current_route.hovered_block == ActiveBlock::Artists,
    );

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Artists",
        &artist_items,
        highlight_state,
        None,
    );
}

pub fn draw_song_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let track_items: Vec<String> = Vec::new();

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::TrackTable,
        current_route.hovered_block == ActiveBlock::TrackTable,
    );

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Tracks",
        &track_items,
        highlight_state,
        None,
    );
}

pub fn draw_home<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_album_list<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let album_items: Vec<String> = Vec::new();

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::AlbumList,
        current_route.hovered_block == ActiveBlock::AlbumList,
    );

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Albums",
        &album_items,
        highlight_state,
        None,
    );
}

pub fn draw_artist_albums<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_playbar<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
}

pub fn draw_play_queue<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let track_items: Vec<String> = Vec::new();

    let current_route = app.get_current_route();

    let highlight_state = (
        current_route.active_block == ActiveBlock::PlayQueue,
        current_route.hovered_block == ActiveBlock::PlayQueue,
    );

    draw_selectable_list(
        f,
        app,
        layout_chunk,
        "Play Queue",
        &track_items,
        highlight_state,
        None,
    );
}
