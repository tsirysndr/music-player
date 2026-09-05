use ratatui::{
    layout::{Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Row, Table},
    Frame,
};

use crate::app::{ActiveBlock, App, Pagination, RouteId, SearchScope, LIBRARY_OPTIONS};

use self::util::{
    centered_rect, centered_rect_absolute, display_track_progress, get_color, get_percentage_width,
    get_track_progress_percentage, millis_to_minutes,
};

pub mod util;

pub enum TableId {
    AlbumList,
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

/// Title for an incrementally loaded table: appends a subtle "N loaded …"
/// marker while more items are available or a page is in flight.
fn paged_title(base: &str, loaded: usize, pagination: &Pagination) -> String {
    if pagination.loading || pagination.has_more {
        format!("{} ({} loaded …)", base, loaded)
    } else {
        base.to_string()
    }
}

pub fn draw_main_layout(f: &mut Frame, app: &App) {
    let margin = util::get_main_layout_margin(app);

    let parent_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(4),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(margin)
        .split(f.area());

    draw_routes(f, app, parent_layout[0]);
    draw_playbar(f, app, parent_layout[1]);
    draw_status_line(f, app, parent_layout[2]);
    draw_hint_bar(f, app, parent_layout[3]);

    if app.search.active {
        draw_search_overlay(f, app);
    }

    if app.show_help {
        draw_help_overlay(f, app);
    }
}

pub fn draw_routes(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(layout_chunk);
    draw_user_block(f, app, chunks[0]);

    let current_route = app.get_current_route();

    match current_route.id {
        RouteId::AlbumTracks => draw_album_table(f, app, chunks[1]),
        RouteId::AlbumList => draw_album_list(f, app, chunks[1]),
        RouteId::Artist => draw_artist_view(f, app, chunks[1]),
        RouteId::TrackTable => draw_song_table(f, app, chunks[1]),
        RouteId::Artists => draw_artist_table(f, app, chunks[1]),
        RouteId::PlayQueue => draw_play_queue(f, app, chunks[1]),
    }
}

pub fn draw_user_block(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(layout_chunk);

    draw_library_block(f, app, chunks[0]);
    draw_playlist_block(f, app, chunks[1]);
}

pub fn draw_library_block(f: &mut Frame, app: &App, layout_chunk: Rect) {
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

pub fn draw_playlist_block(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let playlist_items: Vec<String> = app
        .playlists
        .iter()
        .map(|item| item.name.to_owned())
        .collect();

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

pub fn draw_selectable_list<S>(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    title: &str,
    items: &[S],
    highlight_state: (bool, bool),
    selected_index: Option<usize>,
) where
    S: std::convert::AsRef<str>,
{
    let mut state = ListState::default();
    state.select(selected_index.filter(|&i| i < items.len()));

    let lst_items: Vec<ListItem> = items
        .iter()
        .map(|i| ListItem::new(Span::raw(i.as_ref().to_owned())))
        .collect();

    let list = List::new(lst_items)
        .block(
            Block::default()
                .title(Span::styled(
                    title.to_owned(),
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

pub fn draw_album_table(f: &mut Frame, app: &App, layout_chunk: Rect) {
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
                item.artist.clone(),
                millis_to_minutes((item.duration * 1000.0) as u128),
            ],
        })
        .collect::<Vec<TableItem>>();

    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::AlbumTracks,
        current_route.hovered_block == ActiveBlock::AlbumTracks,
    );

    let title = match &app.selected_album {
        Some(album) if !album.artist.is_empty() => {
            format!("{} by {}", album.title, album.artist)
        }
        Some(album) => album.title.clone(),
        None => "Album".to_string(),
    };

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

pub fn draw_artist_table(f: &mut Frame, app: &App, layout_chunk: Rect) {
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

    let title = paged_title("Artists", items.len(), &app.artist_table.pagination);
    draw_table(
        f,
        app,
        layout_chunk,
        (&title, &header),
        &items,
        app.artist_table.selected_index,
        highlight_state,
    )
}

/// Artist route: the artist's tracks, with the artist's albums listed below
/// when there are any.
pub fn draw_artist_view(f: &mut Frame, app: &App, layout_chunk: Rect) {
    if app.artist_albums.is_empty() {
        draw_artist_song_table(f, app, layout_chunk);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
        .split(layout_chunk);

    draw_artist_song_table(f, app, chunks[0]);
    draw_artist_albums(f, app, chunks[1]);
}

pub fn draw_artist_song_table(f: &mut Frame, app: &App, layout_chunk: Rect) {
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
                item.artist.clone(),
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

    let title = match &app.selected_artist_name {
        Some(name) => format!("Tracks by {}", name),
        None => "Tracks".to_string(),
    };

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

pub fn draw_artist_albums(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let theme = app.user_config.theme;

    let items: Vec<ListItem> = app
        .artist_albums
        .iter()
        .map(|album| {
            let year = if album.year > 0 {
                format!(" ({})", album.year)
            } else {
                String::new()
            };
            ListItem::new(Line::from(vec![
                Span::styled(album.title.clone(), Style::default().fg(theme.text)),
                Span::styled(year, Style::default().fg(theme.inactive)),
            ]))
        })
        .collect();

    let title = format!("Albums ({})", app.artist_albums.len());
    let list = List::new(items)
        .block(
            Block::default()
                .title(Span::styled(title, Style::default().fg(theme.inactive)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.inactive)),
        )
        .style(Style::default().fg(theme.text));
    f.render_widget(list, layout_chunk);
}

pub fn draw_song_table(f: &mut Frame, app: &App, layout_chunk: Rect) {
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
                item.artist.clone(),
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

    let title = paged_title("Tracks", items.len(), &app.track_table.pagination);
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

pub fn draw_album_list(f: &mut Frame, app: &App, layout_chunk: Rect) {
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

    let title = paged_title("Albums", items.len(), &app.album_table.pagination);
    draw_table(
        f,
        app,
        layout_chunk,
        (&title, &header),
        &items,
        app.album_table.selected_index,
        highlight_state,
    )
}

pub fn draw_play_queue(f: &mut Frame, app: &App, layout_chunk: Rect) {
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
                item.artist.clone(),
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

pub fn draw_playbar(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let current_route = app.get_current_route();
    let highlight_state = (
        current_route.active_block == ActiveBlock::PlayBar,
        current_route.hovered_block == ActiveBlock::PlayBar,
    );

    let (title, track_line, progress) = match &app.current_playback_context {
        Some(ctx) if ctx.track.is_some() => {
            let track_item = ctx.track.as_ref().unwrap();
            let play_title = if ctx.is_playing { "Playing" } else { "Paused" };

            let play_bar_text = match &track_item.album {
                Some(album) => format!("{} — {}", track_item.artist, album.title),
                None => track_item.artist.to_string(),
            };

            let track_line = Line::from(vec![
                Span::styled(
                    track_item.title.clone(),
                    Style::default()
                        .fg(app.user_config.theme.selected)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  {}", play_bar_text),
                    Style::default().fg(app.user_config.theme.playbar_text),
                ),
            ]);

            let duration_ms = (track_item.duration * 1000.0) as u32;
            (
                play_title.to_string(),
                track_line,
                Some((app.song_progress_ms, duration_ms)),
            )
        }
        _ => (
            "Not playing".to_string(),
            Line::from(Span::styled(
                "Nothing is playing — press / to find a track",
                Style::default().fg(app.user_config.theme.inactive),
            )),
            None,
        ),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title,
            get_color(highlight_state, app.user_config.theme),
        ))
        .border_style(get_color(highlight_state, app.user_config.theme));
    let inner = block.inner(layout_chunk);
    f.render_widget(block, layout_chunk);

    if inner.height == 0 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    f.render_widget(Paragraph::new(track_line), chunks[0]);

    if let Some((progress_ms, duration_ms)) = progress {
        if chunks.len() > 1 && chunks[1].height > 0 {
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
                    song_progress_label,
                    Style::default().fg(app.user_config.theme.playbar_progress_text),
                ));
            f.render_widget(song_progress, chunks[1]);
        }
    }
}

/// Neovim-style segmented status line.
pub fn draw_status_line(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let theme = app.user_config.theme;

    let (mode, mode_color) = if app.search.active {
        ("SEARCH", theme.statusline_search)
    } else {
        ("NORMAL", theme.statusline_normal)
    };

    let mode_style = Style::default()
        .fg(Color::Black)
        .bg(mode_color)
        .add_modifier(Modifier::BOLD);
    let base_style = Style::default()
        .fg(theme.statusline_fg)
        .bg(theme.statusline_bg);

    let now_playing = match app
        .current_playback_context
        .as_ref()
        .and_then(|ctx| ctx.track.as_ref())
    {
        Some(track) => format!(" {} — {} ", track.artist, track.title),
        None => " — ".to_string(),
    };

    let left_spans = Line::from(vec![
        Span::styled(format!(" {} ", mode), mode_style),
        Span::styled(now_playing, base_style),
    ]);

    // Right side: time, volume, shuffle/repeat, connection.
    let time = match app
        .current_playback_context
        .as_ref()
        .and_then(|ctx| ctx.track.as_ref())
    {
        Some(track) => {
            let duration_ms = (track.duration * 1000.0) as u128;
            format!(
                "{}/{}",
                millis_to_minutes(app.song_progress_ms.min(duration_ms)),
                millis_to_minutes(duration_ms)
            )
        }
        None => "-:--/-:--".to_string(),
    };

    let right = format!(" {}  vol {}%  shuf:off rep:off ", time, app.volume);
    let addr = format!(" {} ", app.server_addr);
    let right_width = (right.chars().count() + addr.chars().count()) as u16;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(right_width)])
        .split(layout_chunk);

    // Fill the whole line with the base background first.
    f.render_widget(
        Paragraph::new(Line::default()).style(base_style),
        layout_chunk,
    );
    f.render_widget(Paragraph::new(left_spans).style(base_style), chunks[0]);
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(right, base_style),
            Span::styled(addr, mode_style),
        ])),
        chunks[1],
    );
}

/// Context-sensitive keybinding hints, displayed under the status line.
pub fn draw_hint_bar(f: &mut Frame, app: &App, layout_chunk: Rect) {
    let hints: &[(&str, &str)] = if app.show_help {
        &[("?/q/esc", "close"), ("j/k", "scroll")]
    } else if app.search.active {
        &[
            ("esc", "close"),
            ("tab", "scope"),
            ("↑/↓ C-j/C-k", "move"),
            ("enter", "play"),
            ("C-u", "clear"),
        ]
    } else {
        &[
            ("?", "help"),
            ("/", "search"),
            ("space", "play/pause"),
            ("n", "next"),
            ("p", "prev"),
            ("<", "-5s"),
            (">", "+5s"),
            ("+/-", "volume"),
            ("z", "queue"),
            ("q", "back/quit"),
        ]
    };

    let theme = app.user_config.theme;
    let mut spans: Vec<Span> = Vec::with_capacity(hints.len() * 2);
    for (key, action) in hints {
        spans.push(Span::styled(
            format!(" {}", key),
            Style::default().fg(theme.hint).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(
            format!(":{} ", action),
            Style::default().fg(theme.inactive),
        ));
    }

    f.render_widget(Paragraph::new(Line::from(spans)), layout_chunk);
}

/// fzf/telescope-style fuzzy finder overlay.
pub fn draw_search_overlay(f: &mut Frame, app: &App) {
    let theme = app.user_config.theme;
    let area = centered_rect(80, 80, f.area());
    if area.width < 5 || area.height < 4 {
        return;
    }

    f.render_widget(Clear, area);

    // Scope tabs in the title, fzf style.
    let mut title_spans = vec![Span::raw(" Search ")];
    for scope in [
        SearchScope::Tracks,
        SearchScope::Albums,
        SearchScope::Artists,
    ] {
        let style = if scope == app.search.scope {
            Style::default()
                .fg(Color::Black)
                .bg(theme.statusline_search)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme.inactive)
        };
        title_spans.push(Span::styled(format!(" {} ", scope.title()), style));
        title_spans.push(Span::raw(" "));
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Line::from(title_spans))
        .border_style(Style::default().fg(theme.active));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner);

    draw_search_results_list(f, app, chunks[0]);

    // Input line at the bottom, like fzf. While the library is still being
    // indexed in the background, pages stream in and results keep growing;
    // a trailing "…" marks the count as partial.
    let status = if app.search.results.is_empty() {
        if app.search.loading {
            " loading library…".to_string()
        } else {
            "  [0]".to_string()
        }
    } else {
        let indexing = if app.search.loading { " …" } else { "" };
        format!(
            "  [{}/{}]{}",
            app.search.selected_index.min(app.search.results.len() - 1) + 1,
            app.search.results.len(),
            indexing
        )
    };
    let prompt = Line::from(vec![
        Span::styled(
            "> ",
            Style::default()
                .fg(theme.active)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(app.search.query.clone()),
        Span::styled(status, Style::default().fg(theme.inactive)),
    ]);
    f.render_widget(Paragraph::new(prompt), chunks[1]);

    let cursor_x = chunks[1].x + 2 + app.search.query.chars().count() as u16;
    f.set_cursor_position(Position::new(
        cursor_x.min(chunks[1].right().saturating_sub(1)),
        chunks[1].y,
    ));
}

/// The live-filtered result list with fuzzy match positions highlighted.
fn draw_search_results_list(f: &mut Frame, app: &App, area: Rect) {
    let theme = app.user_config.theme;

    if app.search.results.is_empty() {
        let msg = if app.search.loading {
            "Loading library…"
        } else if app.search.query.is_empty() {
            "Type to search your library"
        } else {
            "No matches"
        };
        f.render_widget(
            Paragraph::new(Span::styled(msg, Style::default().fg(theme.inactive))),
            area,
        );
        return;
    }

    let height = area.height as usize;
    let selected = app.search.selected_index.min(app.search.results.len() - 1);
    let offset = selected.saturating_sub(height.saturating_sub(1));

    let lines: Vec<Line> = app
        .search
        .results
        .iter()
        .enumerate()
        .skip(offset)
        .take(height)
        .map(|(i, result)| {
            let is_selected = i == selected;
            let base_style = if is_selected {
                Style::default()
                    .fg(theme.selected)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.text)
            };
            let match_style = base_style
                .fg(theme.statusline_search)
                .add_modifier(Modifier::BOLD);

            let mut spans = vec![Span::styled(
                if is_selected { "▌ " } else { "  " },
                base_style,
            )];

            // Group consecutive chars by matched/unmatched to keep span count low.
            let mut current = String::new();
            let mut current_matched = false;
            for (char_idx, c) in result.display.chars().enumerate() {
                let matched = result.indices.binary_search(&(char_idx as u32)).is_ok();
                if matched != current_matched && !current.is_empty() {
                    spans.push(Span::styled(
                        std::mem::take(&mut current),
                        if current_matched {
                            match_style
                        } else {
                            base_style
                        },
                    ));
                }
                current_matched = matched;
                current.push(c);
            }
            if !current.is_empty() {
                spans.push(Span::styled(
                    current,
                    if current_matched {
                        match_style
                    } else {
                        base_style
                    },
                ));
            }

            Line::from(spans)
        })
        .collect();

    f.render_widget(Paragraph::new(Text::from(lines)), area);
}

/// Centered help modal listing every keybinding, grouped by category.
pub fn draw_help_overlay(f: &mut Frame, app: &App) {
    let theme = app.user_config.theme;
    let area = centered_rect_absolute(62, 28, f.area());
    if area.width < 10 || area.height < 4 {
        return;
    }

    f.render_widget(Clear, area);

    let key_style = Style::default().fg(theme.hint).add_modifier(Modifier::BOLD);
    let group_style = Style::default()
        .fg(theme.active)
        .add_modifier(Modifier::BOLD);
    let text_style = Style::default().fg(theme.text);

    let mut lines: Vec<Line> = vec![];
    for (group, entries) in help_entries() {
        lines.push(Line::from(Span::styled(group, group_style)));
        for (key, action) in entries {
            lines.push(Line::from(vec![
                Span::styled(format!("  {:<14}", key), key_style),
                Span::styled(action, text_style),
            ]));
        }
        lines.push(Line::default());
    }

    let paragraph = Paragraph::new(Text::from(lines))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(" Help ", group_style))
                .border_style(Style::default().fg(theme.active)),
        )
        .scroll((app.help_scroll, 0));
    f.render_widget(paragraph, area);
}

/// All keybindings, grouped by category. Used by the `?` help dialog.
pub fn help_entries() -> Vec<(&'static str, Vec<(&'static str, &'static str)>)> {
    vec![
        (
            "Global",
            vec![
                ("?", "Toggle this help dialog"),
                ("/", "Open fuzzy search"),
                ("q", "Go back / quit"),
                ("Ctrl-c", "Quit"),
                ("Esc", "Leave the active block"),
            ],
        ),
        (
            "Navigation",
            vec![
                ("h/j/k/l, arrows", "Move between and within blocks"),
                ("H / M / L", "Jump to top / middle / bottom of a list"),
                ("Enter", "Select the hovered block or item"),
                ("", "Long lists load in pages as you scroll; M/L jump"),
                ("", "within what is loaded so far"),
            ],
        ),
        (
            "Playback",
            vec![
                ("Space", "Play / pause"),
                ("n", "Next track"),
                ("p", "Previous track"),
                ("<", "Seek backwards 5s"),
                (">", "Seek forwards 5s"),
                ("+ / -", "Volume up / down"),
                ("z", "Add selected track to the queue"),
            ],
        ),
        (
            "Search",
            vec![
                ("Type", "Fuzzy-filter the library"),
                ("Tab", "Switch scope (Tracks / Albums / Artists)"),
                ("Up/Down, C-j/C-k", "Move selection"),
                ("Enter", "Play / open selection"),
                ("Ctrl-u", "Clear the query"),
                ("Esc", "Close search"),
            ],
        ),
    ]
}

fn draw_table(
    f: &mut Frame,
    app: &App,
    layout_chunk: Rect,
    table_layout: (&str, &TableHeader), // (title, header columns)
    items: &[TableItem], // The nested vector must have the same length as the `header_columns`
    selected_index: usize,
    highlight_state: (bool, bool),
) {
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
            TableId::PlayQueue => {
                if let Some(current_playback) = &app.current_playback_context {
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
            }
            TableId::Song => {
                if let Some(current_playback) = &app.current_playback_context {
                    if let Some(title_idx) = header.get_index(ColumnId::Title) {
                        if let Some(track) = &current_playback.track {
                            if track.id == item.id {
                                formatted_row[title_idx] =
                                    format!("▶ {}", &formatted_row[title_idx]);
                                style = Style::default()
                                    .fg(app.user_config.theme.active)
                                    .add_modifier(Modifier::BOLD);
                            }
                        }
                    }
                }
            }
            _ => {}
        };

        // Return row styled data
        Row::new(formatted_row).style(style)
    });

    let widths = header
        .items
        .iter()
        .map(|h| Constraint::Length(h.width))
        .collect::<Vec<Constraint>>();

    let table = Table::new(rows, widths)
        .header(
            Row::new(header.items.iter().map(|h| h.text))
                .style(Style::default().fg(app.user_config.theme.header)),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(app.user_config.theme.text))
                .title(Span::styled(
                    title.to_owned(),
                    get_color(highlight_state, app.user_config.theme),
                ))
                .border_style(get_color(highlight_state, app.user_config.theme)),
        )
        .style(Style::default().fg(app.user_config.theme.text));
    f.render_widget(table, layout_chunk);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{
        AlbumTable, App, ArtistTable, LibraryCache, PagedCollection, PlaylistItem, TrackTable,
        LOAD_MORE_THRESHOLD,
    };
    use crate::network::IoEvent;
    use music_player_server::api::{
        metadata::v1alpha1::{Album, Artist, Track},
        music::v1alpha1::GetCurrentlyPlayingSongResponse,
    };
    use ratatui::{backend::TestBackend, Terminal};
    use std::sync::mpsc;

    fn sample_track(id: &str, title: &str) -> Track {
        Track {
            id: id.to_string(),
            title: title.to_string(),
            artist: "Daft Punk".to_string(),
            duration: 240.0,
            track_number: 1,
            album: Some(Album {
                id: "album-1".to_string(),
                title: "Discovery".to_string(),
                artist: "Daft Punk".to_string(),
                year: 2001,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    fn make_app() -> (App, mpsc::Receiver<crate::network::IoEvent>) {
        let (tx, rx) = mpsc::channel();
        (App::new(tx), rx)
    }

    fn draw(app: &App) {
        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw_main_layout(f, app)).unwrap();
    }

    #[test]
    fn renders_every_route_with_empty_state() {
        // The album-tracks route used to index into an empty album table and
        // panic; drawing every route with a completely empty App covers that.
        let routes = [
            (RouteId::TrackTable, ActiveBlock::TrackTable),
            (RouteId::AlbumList, ActiveBlock::AlbumList),
            (RouteId::AlbumTracks, ActiveBlock::AlbumTracks),
            (RouteId::Artists, ActiveBlock::Artists),
            (RouteId::Artist, ActiveBlock::ArtistBlock),
            (RouteId::PlayQueue, ActiveBlock::PlayQueue),
        ];
        for (route, block) in routes {
            let (mut app, _rx) = make_app();
            app.push_navigation_stack(route, block);
            draw(&app);
        }
    }

    #[test]
    fn renders_every_route_with_data() {
        let routes = [
            (RouteId::TrackTable, ActiveBlock::TrackTable),
            (RouteId::AlbumList, ActiveBlock::AlbumList),
            (RouteId::AlbumTracks, ActiveBlock::AlbumTracks),
            (RouteId::Artists, ActiveBlock::Artists),
            (RouteId::Artist, ActiveBlock::ArtistBlock),
            (RouteId::PlayQueue, ActiveBlock::PlayQueue),
        ];
        for (route, block) in routes {
            let (mut app, _rx) = make_app();
            app.track_table = TrackTable {
                tracks: vec![
                    sample_track("t1", "One More Time"),
                    sample_track("t2", "Aerodynamic"),
                ],
                selected_index: 1,
                pagination: Default::default(),
            };
            app.album_table = AlbumTable {
                albums: vec![Album {
                    id: "album-1".to_string(),
                    title: "Discovery".to_string(),
                    artist: "Daft Punk".to_string(),
                    year: 2001,
                    ..Default::default()
                }],
                selected_index: 0,
                pagination: Default::default(),
            };
            app.artist_table = ArtistTable {
                artists: vec![Artist {
                    id: "artist-1".to_string(),
                    name: "Daft Punk".to_string(),
                    ..Default::default()
                }],
                selected_index: 0,
                pagination: Default::default(),
            };
            app.artist_albums = app.album_table.albums.clone();
            app.selected_album = app.album_table.albums.first().cloned();
            app.selected_artist_name = Some("Daft Punk".to_string());
            app.playlists = vec![PlaylistItem {
                id: "p1".to_string(),
                name: "Favorites".to_string(),
            }];
            app.selected_playlist_index = Some(0);
            app.current_playback_context = Some(GetCurrentlyPlayingSongResponse {
                track: Some(sample_track("t1", "One More Time")),
                index: 0,
                position_ms: 1000,
                is_playing: true,
            });
            app.song_progress_ms = 1000;
            app.push_navigation_stack(route, block);
            draw(&app);
        }
    }

    #[test]
    fn dispatches_load_more_near_end_of_paged_collection() {
        let (mut app, rx) = make_app();
        let tracks: Vec<Track> = (0..200)
            .map(|i| sample_track(&format!("t{}", i), &format!("Track {}", i)))
            .collect();
        app.track_table = TrackTable {
            selected_index: 200 - LOAD_MORE_THRESHOLD,
            tracks,
            pagination: Pagination {
                next_offset: 200,
                has_more: true,
                loading: false,
            },
        };

        app.maybe_load_more(PagedCollection::Tracks);
        assert!(app.track_table.pagination.loading);
        assert!(matches!(
            rx.try_recv(),
            Ok(IoEvent::LoadMore(PagedCollection::Tracks))
        ));

        // A second call while the page is in flight must not dispatch again.
        app.maybe_load_more(PagedCollection::Tracks);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn does_not_load_more_far_from_end_or_when_exhausted() {
        let (mut app, rx) = make_app();
        let tracks: Vec<Track> = (0..200)
            .map(|i| sample_track(&format!("t{}", i), &format!("Track {}", i)))
            .collect();

        // Far from the end of a partially loaded collection: no request.
        app.track_table = TrackTable {
            selected_index: 0,
            tracks: tracks.clone(),
            pagination: Pagination {
                next_offset: 200,
                has_more: true,
                loading: false,
            },
        };
        app.maybe_load_more(PagedCollection::Tracks);
        assert!(!app.track_table.pagination.loading);
        assert!(rx.try_recv().is_err());

        // At the end of an exhausted collection: no request either.
        app.track_table = TrackTable {
            selected_index: 199,
            tracks,
            pagination: Pagination::default(),
        };
        app.maybe_load_more(PagedCollection::Tracks);
        assert!(rx.try_recv().is_err());

        // Empty collections never panic or request anything.
        app.track_table = TrackTable::default();
        app.album_table = AlbumTable::default();
        app.artist_table = ArtistTable::default();
        app.maybe_load_more(PagedCollection::Tracks);
        app.maybe_load_more(PagedCollection::Albums);
        app.maybe_load_more(PagedCollection::Artists);
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn paged_title_marks_partially_loaded_tables() {
        let partial = Pagination {
            next_offset: 200,
            has_more: true,
            loading: false,
        };
        assert_eq!(
            paged_title("Tracks", 200, &partial),
            "Tracks (200 loaded …)"
        );

        let in_flight = Pagination {
            next_offset: 200,
            has_more: false,
            loading: true,
        };
        assert_eq!(
            paged_title("Tracks", 200, &in_flight),
            "Tracks (200 loaded …)"
        );

        assert_eq!(paged_title("Tracks", 2, &Pagination::default()), "Tracks");
    }

    #[test]
    fn renders_partially_loaded_tables() {
        let routes = [
            (RouteId::TrackTable, ActiveBlock::TrackTable),
            (RouteId::AlbumList, ActiveBlock::AlbumList),
            (RouteId::Artists, ActiveBlock::Artists),
        ];
        for (route, block) in routes {
            let (mut app, _rx) = make_app();
            let partial = || Pagination {
                next_offset: 1,
                has_more: true,
                loading: true,
            };
            app.track_table = TrackTable {
                tracks: vec![sample_track("t1", "One More Time")],
                selected_index: 0,
                pagination: partial(),
            };
            app.album_table = AlbumTable {
                albums: vec![Album {
                    id: "album-1".to_string(),
                    title: "Discovery".to_string(),
                    ..Default::default()
                }],
                selected_index: 0,
                pagination: partial(),
            };
            app.artist_table = ArtistTable {
                artists: vec![Artist {
                    id: "artist-1".to_string(),
                    name: "Daft Punk".to_string(),
                    ..Default::default()
                }],
                selected_index: 0,
                pagination: partial(),
            };
            app.push_navigation_stack(route, block);
            draw(&app);
        }
    }

    #[test]
    fn renders_search_overlay_and_filters() {
        let (mut app, _rx) = make_app();
        app.library_cache = Some(LibraryCache {
            tracks: vec![
                sample_track("t1", "One More Time"),
                sample_track("t2", "Harder Better Faster Stronger"),
            ],
            albums: vec![],
            artists: vec![],
        });
        app.open_search();
        assert_eq!(app.search.results.len(), 2);
        app.search.query = "harder".to_string();
        app.refresh_search_results();
        assert_eq!(app.search.results.len(), 1);
        assert!(!app.search.results[0].indices.is_empty());
        draw(&app);
    }

    #[test]
    fn renders_help_overlay() {
        let (mut app, _rx) = make_app();
        app.show_help = true;
        app.help_scroll = 2;
        draw(&app);
    }

    #[test]
    fn renders_in_tiny_terminal() {
        let (mut app, _rx) = make_app();
        app.show_help = true;
        app.search.active = true;
        let backend = TestBackend::new(10, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|f| draw_main_layout(f, &app)).unwrap();
    }
}
