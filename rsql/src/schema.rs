//! What a filter is allowed to mention, and how each field reaches SQL.
//!
//! A filter arrives as user input, so nothing in it may ever be interpolated
//! into SQL. A field is only usable if it appears here, and it contributes a
//! *fixed* column expression written by us; every value the user typed leaves
//! as a bound parameter. That is the whole injection story.

/// How a field's values are compared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldKind {
    Text,
    Integer,
    /// Stored as a number but written as `true` / `false` / `yes` / `1`.
    Boolean,
    /// A timestamp column. Accepts an ISO date, or a relative age like `30d`
    /// / `6m` / `1y` which resolves against "now" at compile time.
    Timestamp,
}

/// One filterable field: the name a user writes, and the SQL behind it.
#[derive(Clone, Debug)]
pub struct Field {
    pub name: &'static str,
    /// The SQL expression this field compares against. Written by us, never
    /// by the user — see the module docs.
    pub column: &'static str,
    pub kind: FieldKind,
    /// A short description, surfaced by the UIs that offer field pickers.
    pub label: &'static str,
}

impl Field {
    const fn new(
        name: &'static str,
        column: &'static str,
        kind: FieldKind,
        label: &'static str,
    ) -> Self {
        Self {
            name,
            column,
            kind,
            label,
        }
    }
}

/// The set of fields one kind of query may filter on, plus the FROM clause
/// that makes those columns resolvable.
#[derive(Clone, Debug)]
pub struct Schema {
    pub name: &'static str,
    /// `FROM` body — the base table and any joins the fields depend on.
    pub from: &'static str,
    /// The identifier column the query selects.
    pub id_column: &'static str,
    pub fields: &'static [Field],
}

impl Schema {
    pub fn field(&self, name: &str) -> Option<&Field> {
        let name = name.to_ascii_lowercase();
        self.fields.iter().find(|f| f.name == name)
    }

    /// Field names, for "did you mean" hints and UI pickers.
    pub fn field_names(&self) -> Vec<&'static str> {
        self.fields.iter().map(|f| f.name).collect()
    }
}

/// Tracks, with their album and artist joined in so a filter can say
/// `album==...` without the caller knowing the schema.
///
/// `track_stats` is a LEFT JOIN: a track nobody has played yet has no row
/// there, and `play_count==0` still has to match it — hence the COALESCE.
pub const TRACKS: Schema = Schema {
    name: "tracks",
    from: "track \
           LEFT JOIN album ON album.id = track.album_id \
           LEFT JOIN track_stats ON track_stats.track_id = track.id",
    id_column: "track.id",
    fields: &[
        Field::new("title", "track.title", FieldKind::Text, "Title"),
        Field::new("artist", "track.artist", FieldKind::Text, "Artist"),
        Field::new("album", "album.title", FieldKind::Text, "Album"),
        Field::new("genre", "track.genre", FieldKind::Text, "Genre"),
        Field::new("year", "track.year", FieldKind::Integer, "Year"),
        Field::new("track", "track.track", FieldKind::Integer, "Track number"),
        Field::new("duration", "track.duration", FieldKind::Integer, "Duration"),
        Field::new("bitrate", "track.bitrate", FieldKind::Integer, "Bitrate"),
        Field::new(
            "samplerate",
            "track.sample_rate",
            FieldKind::Integer,
            "Sample rate",
        ),
        Field::new("uri", "track.uri", FieldKind::Text, "File path"),
        Field::new(
            "liked",
            "EXISTS (SELECT 1 FROM rocksky_like WHERE rocksky_like.track_id = track.id)",
            FieldKind::Boolean,
            "Liked",
        ),
        Field::new(
            "playcount",
            "COALESCE(track_stats.play_count, 0)",
            FieldKind::Integer,
            "Play count",
        ),
        Field::new(
            "skipcount",
            "COALESCE(track_stats.skip_count, 0)",
            FieldKind::Integer,
            "Skip count",
        ),
        Field::new(
            "lastplayed",
            "track_stats.last_played",
            FieldKind::Timestamp,
            "Last played",
        ),
        Field::new(
            "added",
            "track.created_at",
            FieldKind::Timestamp,
            "Date added",
        ),
    ],
};

pub const ALBUMS: Schema = Schema {
    name: "albums",
    from: "album",
    id_column: "album.id",
    fields: &[
        Field::new("title", "album.title", FieldKind::Text, "Title"),
        Field::new("artist", "album.artist", FieldKind::Text, "Artist"),
        Field::new("year", "album.year", FieldKind::Integer, "Year"),
        Field::new("cover", "album.cover", FieldKind::Text, "Cover"),
    ],
};

pub const ARTISTS: Schema = Schema {
    name: "artists",
    from: "artist",
    id_column: "artist.id",
    fields: &[Field::new("name", "artist.name", FieldKind::Text, "Name")],
};

pub const PLAYLISTS: Schema = Schema {
    name: "playlists",
    from: "playlist",
    id_column: "playlist.id",
    fields: &[
        Field::new("name", "playlist.name", FieldKind::Text, "Name"),
        Field::new(
            "description",
            "playlist.description",
            FieldKind::Text,
            "Description",
        ),
        Field::new(
            "smart",
            "playlist.is_smart",
            FieldKind::Boolean,
            "Smart playlist",
        ),
        Field::new(
            "created",
            "playlist.created_at",
            FieldKind::Timestamp,
            "Created",
        ),
    ],
};

/// Every schema, for callers that resolve one by name (the extension host and
/// the GraphQL layer both do).
pub const ALL: &[&Schema] = &[&TRACKS, &ALBUMS, &ARTISTS, &PLAYLISTS];

/// Look a schema up by the name a caller passes in ("tracks", "albums", …).
pub fn by_name(name: &str) -> Option<&'static Schema> {
    let name = name.to_ascii_lowercase();
    ALL.iter().copied().find(|schema| schema.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_fields_case_insensitively() {
        assert!(TRACKS.field("Artist").is_some());
        assert!(TRACKS.field("PLAYCOUNT").is_some());
        assert!(TRACKS.field("nope").is_none());
    }

    #[test]
    fn schemas_are_reachable_by_name() {
        assert_eq!(by_name("tracks").unwrap().name, "tracks");
        assert_eq!(by_name("Albums").unwrap().name, "albums");
        assert!(by_name("songs").is_none());
    }

    /// Field names are what users type; a duplicate would silently shadow.
    #[test]
    fn field_names_are_unique_and_lowercase() {
        for schema in ALL {
            let mut names = schema.field_names();
            names.sort_unstable();
            let count = names.len();
            names.dedup();
            assert_eq!(names.len(), count, "{} has a duplicate field", schema.name);
            for name in schema.field_names() {
                assert_eq!(name, name.to_ascii_lowercase(), "{name} is not lowercase");
            }
        }
    }
}
