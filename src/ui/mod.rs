use music_player_playback::player::RepeatState;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Span, Text},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Row, Table},
    Frame,
};

use crate::{
    app::{ActiveBlock, App, RouteId, LIBRARY_OPTIONS},
    ui::util::SMALL_TERMINAL_WIDTH,
};

use self::util::{
    create_artist_string, display_track_progress, get_color, get_percentage_width,
    get_track_progress_percentage, millis_to_minutes,
};

pub mod util;

pub enum TableId {
    Album,
    AlbumList,
    Artist,
    ArtistList,
    Song,
    PlayQueue,
}

#[derive(Default, PartialEq)]
pub enum ColumnId {
    #[default]
    None,
    Title,
    Artist,
    Album,
}

pub struct TableHeader<'a> {
    id: TableId,
    items: Vec<TableHeaderItem<'a>>,
}

impl TableHeader<'_> {
    pub fn get_index(&self, id: ColumnId) -> Option<usize> {
        self.items.iter().position(|item| item.id == id)
    }
}

#[derive(Default)]
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

    // Currently playing
    draw_playbar(f, app, parent_layout[2]);
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
        RouteId::Artist => draw_artist_song_table(f, app, chunks[1]),
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
    let header = TableHeader {
        id: TableId::Song,
        items: vec![
            TableHeaderItem {
                text: "#",
                width: get_percentage_width(layout_chunk.width, 0.1),
                ..Default::default()
            },
            TableHeaderItem {
                id: ColumnId::Title,
                text: "Title",
                width: get_percentage_width(layout_chunk.width, 0.3),
            },
            TableHeaderItem {
                text: "Artist",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Duration",
                width: get_percentage_width(layout_chunk.width, 0.2),
                ..Default::default()
            },
        ],
    };

    let items = app
        .track_table
        .tracks
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![
                item.track_number.to_string(),
                item.title.clone(),
                item.artists
                    .iter()
                    .map(|a| a.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(", "),
                millis_to_minutes((item.duration * 1000.0) as u128),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::AlbumTracks,
        current_route.hovered_block == ActiveBlock::AlbumTracks,
    );

    let title = format!(
        "{} by {}",
        app.album_table.albums[app.album_table.selected_index].title,
        app.album_table.albums[app.album_table.selected_index].artist
    );

    draw_table(
        f,
        app,
        layout_chunk,
        (&title, &header),
        &items,
        app.track_table.selected_index,
        highlight_state,
    )
}

pub fn draw_artist_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let header = TableHeader {
        id: TableId::ArtistList,
        items: vec![TableHeaderItem {
            id: ColumnId::Artist,
            text: "Name",
            width: get_percentage_width(layout_chunk.width, 1.0),
        }],
    };

    let items = app
        .artist_table
        .artists
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![item.name.clone()],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::Artists,
        current_route.hovered_block == ActiveBlock::Artists,
    );

    draw_table(
        f,
        app,
        layout_chunk,
        ("Artists", &header),
        &items,
        app.artist_table.selected_index,
        highlight_state,
    )
}

pub fn draw_artist_song_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let header = TableHeader {
        id: TableId::Song,
        items: vec![
            TableHeaderItem {
                id: ColumnId::Title,
                text: "Title",
                width: get_percentage_width(layout_chunk.width, 0.3),
            },
            TableHeaderItem {
                text: "Artist",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Album",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Duration",
                width: get_percentage_width(layout_chunk.width, 0.1),
                ..Default::default()
            },
        ],
    };

    let items = app
        .track_table
        .tracks
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![
                item.title.clone(),
                item.artists
                    .iter()
                    .map(|a| a.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(", "),
                item.album.clone().unwrap_or_default().title,
                millis_to_minutes((item.duration * 1000.0) as u128),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::ArtistBlock,
        current_route.hovered_block == ActiveBlock::ArtistBlock,
    );

    draw_table(
        f,
        app,
        layout_chunk,
        ("Tracks", &header),
        &items,
        app.track_table.selected_index,
        highlight_state,
    )
}

pub fn draw_song_table<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let header = TableHeader {
        id: TableId::Song,
        items: vec![
            TableHeaderItem {
                id: ColumnId::Title,
                text: "Title",
                width: get_percentage_width(layout_chunk.width, 0.3),
            },
            TableHeaderItem {
                text: "Artist",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Album",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Duration",
                width: get_percentage_width(layout_chunk.width, 0.1),
                ..Default::default()
            },
        ],
    };

    let items = app
        .track_table
        .tracks
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![
                item.title.clone(),
                item.artists
                    .iter()
                    .map(|a| a.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(", "),
                item.album.clone().unwrap_or_default().title,
                millis_to_minutes((item.duration * 1000.0) as u128),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::TrackTable,
        current_route.hovered_block == ActiveBlock::TrackTable,
    );

    draw_table(
        f,
        app,
        layout_chunk,
        ("Tracks", &header),
        &items,
        app.track_table.selected_index,
        highlight_state,
    )
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
    let header = TableHeader {
        id: TableId::AlbumList,
        items: vec![
            TableHeaderItem {
                id: ColumnId::Title,
                text: "Title",
                width: get_percentage_width(layout_chunk.width, 0.4),
            },
            TableHeaderItem {
                text: "Artist",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Year",
                width: get_percentage_width(layout_chunk.width, 0.1),
                ..Default::default()
            },
        ],
    };

    let items = app
        .album_table
        .albums
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![
                item.title.clone(),
                item.artist.clone(),
                item.year.to_string(),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::AlbumList,
        current_route.hovered_block == ActiveBlock::AlbumList,
    );

    draw_table(
        f,
        app,
        layout_chunk,
        ("Albums", &header),
        &items,
        app.album_table.selected_index,
        highlight_state,
    )
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
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(50),
                Constraint::Percentage(25),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .margin(1)
        .split(layout_chunk);

    if let Some(current_playback_context) = &app.current_playback_context {
        if let Some(track_item) = &current_playback_context.track {
            let play_title = if current_playback_context.is_playing {
                "Playing"
            } else {
                "Paused"
            };

            let shuffle_text = "Off";
            /*
            if current_playback_context.shuffle_state {
                "On"
            } else {
                "Off"
            };
            */

            let repeat_text = "Off";
            /*
            match current_playback_context.repeat_state {
                RepeatState::Off => "Off",
                RepeatState::One => "On",
                RepeatState::All => "All",
            };
            */

            let title = format!(
                "{:-7} (Shuffle: {:-3} | Repeat: {:})",
                play_title, shuffle_text, repeat_text,
            );

            let current_route = app.get_current_route();
            let highlight_state = (
                current_route.active_block == ActiveBlock::PlayBar,
                current_route.hovered_block == ActiveBlock::PlayBar,
            );

            let title_block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    &title,
                    get_color(highlight_state, app.user_config.theme),
                ))
                .border_style(get_color(highlight_state, app.user_config.theme));

            f.render_widget(title_block, layout_chunk);

            let track_name = track_item.title.clone();
            let play_bar_text = match track_item.album.clone() {
                Some(album) => format!(
                    "{} - {}",
                    create_artist_string(&track_item.artists),
                    album.title
                ),
                None => create_artist_string(&track_item.artists),
            };

            let lines = Text::from(Span::styled(
                play_bar_text,
                Style::default().fg(app.user_config.theme.playbar_text),
            ));

            let artist = Paragraph::new(lines)
                .style(Style::default().fg(app.user_config.theme.playbar_text))
                .block(
                    Block::default().title(Span::styled(
                        &track_name,
                        Style::default()
                            .fg(app.user_config.theme.selected)
                            .add_modifier(Modifier::BOLD),
                    )),
                );
            f.render_widget(artist, chunks[0]);

            let progress_ms = match app.seek_ms {
                Some(seek_ms) => seek_ms,
                None => app.song_progress_ms,
            };

            let duration_ms = (track_item.duration * 1000.0) as u32;

            let perc = get_track_progress_percentage(progress_ms, duration_ms);

            let song_progress_label = display_track_progress(progress_ms, duration_ms);
            let modifier = if app.user_config.behavior.enable_text_emphasis {
                Modifier::ITALIC | Modifier::BOLD
            } else {
                Modifier::empty()
            };
            let song_progress = Gauge::default()
                .gauge_style(
                    Style::default()
                        .fg(app.user_config.theme.playbar_progress)
                        .bg(app.user_config.theme.playbar_background)
                        .add_modifier(modifier),
                )
                .percent(perc)
                .label(Span::styled(
                    &song_progress_label,
                    Style::default().fg(app.user_config.theme.playbar_progress_text),
                ));
            f.render_widget(song_progress, chunks[2]);
        }
    }
}

fn draw_table<B>(
    f: &mut Frame<B>,
    app: &App,
    layout_chunk: Rect,
    table_layout: (&str, &TableHeader), // (title, header colums)
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    selected_index: usize,
    highlight_state: (bool, bool),
) where
    B: Backend,
{
    let selected_style =
        get_color(highlight_state, app.user_config.theme).add_modifier(Modifier::BOLD);

    let (title, header) = table_layout;

    // Make sure that the selected item is visible on the page. Need to add some rows of padding
    // to chunk height for header and header space to get a true table height
    let padding = 5;
    let offset = layout_chunk
        .height
        .checked_sub(padding)
        .and_then(|height| selected_index.checked_sub(height as usize))
        .unwrap_or(0);

    let rows = items.iter().skip(offset).enumerate().map(|(i, item)| {
        let mut formatted_row = item.format.clone();
        let mut style = Style::default().fg(app.user_config.theme.text); // default styling

        // Next check if the item is under selection.
        if Some(i) == selected_index.checked_sub(offset) {
            style = selected_style;
        }

        // if table displays songs
        match header.id {
            TableId::PlayQueue => match app.current_playback_context.clone() {
                Some(current_playback) => {
                    if let Some(title_idx) = header.get_index(ColumnId::Title) {
                        if let Some(track_playing_offset_index) =
                            current_playback.index.checked_sub(offset as u32)
                        {
                            if track_playing_offset_index == (i as u32 + 1) {
                                formatted_row[title_idx] =
                                    format!("▶ {}", &formatted_row[title_idx]);
                                style = Style::default()
                                    .fg(app.user_config.theme.active)
                                    .add_modifier(Modifier::BOLD);
                            }
                        }
                    }
                }
                _ => {}
            },
            TableId::Song | TableId::Album => match app.current_playback_context.clone() {
                Some(current_playback) => {
                    if let Some(title_idx) = header.get_index(ColumnId::Title) {
                        match current_playback.track {
                            Some(track) => {
                                if track.id == item.id {
                                    formatted_row[title_idx] =
                                        format!("▶ {}", &formatted_row[title_idx]);
                                    style = Style::default()
                                        .fg(app.user_config.theme.active)
                                        .add_modifier(Modifier::BOLD);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };

        // Return row styled data
        Row::new(formatted_row).style(style)
    });

    let widths = header
        .items
        .iter()
        .map(|h| Constraint::Length(h.width))
        .collect::<Vec<tui::layout::Constraint>>();

    let table = Table::new(rows)
        .header(
            Row::new(header.items.iter().map(|h| h.text))
                .style(Style::default().fg(app.user_config.theme.header)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.user_config.theme.text))
                .title(Span::styled(
                    title,
                    get_color(highlight_state, app.user_config.theme),
                ))
                .border_style(get_color(highlight_state, app.user_config.theme)),
        )
        .style(Style::default().fg(app.user_config.theme.text))
        .widths(&widths);
    f.render_widget(table, layout_chunk);
}

pub fn draw_play_queue<B>(f: &mut Frame<B>, app: &App, layout_chunk: Rect)
where
    B: Backend,
{
    let header = TableHeader {
        id: TableId::PlayQueue,
        items: vec![
            TableHeaderItem {
                id: ColumnId::Title,
                text: "Title",
                width: get_percentage_width(layout_chunk.width, 0.3),
            },
            TableHeaderItem {
                text: "Artist",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Album",
                width: get_percentage_width(layout_chunk.width, 0.3),
                ..Default::default()
            },
            TableHeaderItem {
                text: "Duration",
                width: get_percentage_width(layout_chunk.width, 0.1),
                ..Default::default()
            },
        ],
    };

    let items = app
        .track_table
        .tracks
        .iter()
        .map(|item| TableItem {
            id: item.id.clone(),
            format: vec![
                item.title.clone(),
                item.artists
                    .iter()
                    .map(|a| a.name.to_owned())
                    .collect::<Vec<String>>()
                    .join(", "),
                item.album.clone().unwrap_or_default().title,
                millis_to_minutes((item.duration * 1000.0) as u128),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::PlayQueue,
        current_route.hovered_block == ActiveBlock::PlayQueue,
    );

    draw_table(
        f,
        app,
        layout_chunk,
        ("Play Queue", &header),
        &items,
        app.track_table.selected_index,
        highlight_state,
    )
}
