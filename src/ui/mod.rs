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

pub fn draw_main_layout() {}

pub fn draw_input() {}

pub fn draw_library_block() {}

pub fn draw_playlist_block() {}

pub fn draw_search_results() {}

pub fn draw_selectable_list() {}

pub fn draw_album_table() {}

pub fn draw_song_table() {}

pub fn draw_home() {}

pub fn draw_album_list() {}

pub fn draw_artist_albums() {}

pub fn draw_playbar() {}

pub fn draw_table() {}
