use music_player_types::types::{Album, Artist, SimplifiedSong, Song};

pub mod album;
pub mod artist;
pub mod track;

#[derive(Clone)]
pub struct Searcher {
    pub track: track::TrackSearcher,
    pub artist: artist::ArtistSearcher,
    pub album: album::AlbumSearcher,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            track: track::TrackSearcher::new(),
            artist: artist::ArtistSearcher::new(),
            album: album::AlbumSearcher::new(),
        }
    }

    pub fn insert_song(&self, song: Song, id: &str) -> tantivy::Result<()> {
        self.track.insert(song, id)
    }

    pub fn insert_artist(&self, artist: Artist) -> tantivy::Result<()> {
        self.artist.insert(artist)
    }

    pub fn insert_album(&self, album: Album) -> tantivy::Result<()> {
        self.album.insert(album)
    }

    pub fn search_artist(&self, term: &str) -> tantivy::Result<Vec<Artist>> {
        self.artist.search(term)
    }

    pub fn search_album(&self, term: &str) -> tantivy::Result<Vec<Album>> {
        self.album.search(term)
    }

    pub fn search_song(&self, term: &str) -> tantivy::Result<Vec<SimplifiedSong>> {
        self.track.search(term)
    }
}
