//! RSQL filters, compiled to SQLite.
//!
//! [RSQL](https://github.com/jirutka/rsql-parser) (a superset of FIQL) is a
//! filter language that survives being typed by a human and pasted into a URL:
//!
//! ```text
//! artist==Radiohead;year=gt=2000
//! genre=in=(rock,jazz);playcount>5
//! (liked==true,playcount>10);lastplayed=lt=30d
//! ```
//!
//! `;` is AND, `,` is OR (AND binds tighter), and parentheses group. Operators
//! come in both FIQL (`=gt=`) and symbolic (`>`) spellings.
//!
//! # Safety
//!
//! A filter is user input and is never interpolated into SQL. Field names are
//! resolved through a [`Schema`] to fixed column expressions written here;
//! every literal is bound as a parameter. See [`schema`] and [`sql`].
//!
//! # Example
//!
//! ```
//! use music_player_rsql::{compile, schema::TRACKS};
//!
//! let filter = compile("genre==rock;year>2000", &TRACKS).unwrap();
//! assert_eq!(
//!     filter.sql,
//!     "(track.genre = ? COLLATE NOCASE AND track.year > ?)"
//! );
//! assert_eq!(filter.params.len(), 2);
//! ```

pub mod ast;
pub mod editor;
pub mod error;
pub mod lexer;
pub mod schema;
pub mod sql;

pub use ast::{parse, Constraint, Node};
pub use editor::{complete, highlight, Completion, Kind, Span, Suggestion, What};
pub use error::{Error, Result};
pub use lexer::Comparison;
pub use schema::{Field, FieldKind, Schema, ALBUMS, ARTISTS, PLAYLISTS, TRACKS};
pub use sql::{compile, compile_at, Filter, Value};

use serde::{Deserialize, Serialize};

/// How a generated list is ordered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortOrder {
    #[default]
    Asc,
    Desc,
}

/// A filter plus the ordering and cap that turn it into a finite list — the
/// three things a smart playlist is made of.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct QuerySpec {
    /// The RSQL filter. Empty matches everything.
    #[serde(default)]
    pub filter: String,
    /// Field to sort by, named as in the schema. `random` shuffles.
    #[serde(default)]
    pub sort_by: Option<String>,
    #[serde(default)]
    pub sort_order: SortOrder,
    /// Maximum rows. `None` is unlimited.
    #[serde(default)]
    pub limit: Option<u32>,
}

/// A ready-to-run statement: SQL selecting the id column, and its parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct Query {
    pub sql: String,
    pub params: Vec<Value>,
}

/// The sort key that shuffles instead of ordering.
pub const SORT_RANDOM: &str = "random";

/// Build the id-selecting query for `spec` against `schema`.
///
/// The result is `SELECT <id> FROM <from> WHERE <filter> ORDER BY … LIMIT …`,
/// which callers run to get the matching ids and then load in full.
pub fn build(spec: &QuerySpec, schema: &Schema) -> Result<Query> {
    build_at(spec, schema, now())
}

/// [`build`], with the reference point for relative ages passed in.
pub fn build_at(spec: &QuerySpec, schema: &Schema, now: i64) -> Result<Query> {
    let mut params = Vec::new();
    let mut sql = format!("SELECT {} FROM {}", schema.id_column, schema.from);

    let filter = spec.filter.trim();
    if !filter.is_empty() {
        let compiled = compile_at(filter, schema, now)?;
        sql.push_str(" WHERE ");
        sql.push_str(&compiled.sql);
        params = compiled.params;
    }

    if let Some(sort_by) = spec
        .sort_by
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        if sort_by.eq_ignore_ascii_case(SORT_RANDOM) {
            sql.push_str(" ORDER BY RANDOM()");
        } else {
            let field = schema.field(sort_by).ok_or_else(|| {
                Error::new(
                    format!(
                        "cannot sort by '{sort_by}' — try one of: {}, {SORT_RANDOM}",
                        schema.field_names().join(", ")
                    ),
                    0,
                )
            })?;
            // Rows with no value sort last either way: a smart playlist of
            // "least played" should not open with every untagged track.
            sql.push_str(&format!(
                " ORDER BY ({} IS NULL), {} {}",
                field.column,
                field.column,
                match spec.sort_order {
                    SortOrder::Asc => "ASC",
                    SortOrder::Desc => "DESC",
                }
            ));
        }
    }

    if let Some(limit) = spec.limit.filter(|n| *n > 0) {
        // A bare integer, not a bound parameter: some SQLite bindings will not
        // bind LIMIT, and the value is a u32 we produced, not user text.
        sql.push_str(&format!(" LIMIT {limit}"));
    }

    Ok(Query { sql, params })
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// Check a filter without building a query — what a UI calls as the user types.
/// `Ok(())` means it parses and every field exists in `schema`.
pub fn validate(filter: &str, schema: &Schema) -> Result<()> {
    if filter.trim().is_empty() {
        return Ok(());
    }
    compile(filter, schema).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_700_000_000;

    #[test]
    fn builds_a_full_query() {
        let spec = QuerySpec {
            filter: "genre==rock".into(),
            sort_by: Some("playcount".into()),
            sort_order: SortOrder::Desc,
            limit: Some(25),
        };
        let query = build_at(&spec, &TRACKS, NOW).unwrap();
        assert_eq!(
            query.sql,
            "SELECT track.id FROM track \
             LEFT JOIN album ON album.id = track.album_id \
             LEFT JOIN track_stats ON track_stats.track_id = track.id \
             WHERE track.genre = ? COLLATE NOCASE \
             ORDER BY (COALESCE(track_stats.play_count, 0) IS NULL), \
             COALESCE(track_stats.play_count, 0) DESC LIMIT 25"
        );
        assert_eq!(query.params, vec![Value::Text("rock".into())]);
    }

    #[test]
    fn an_empty_filter_matches_everything() {
        let spec = QuerySpec {
            limit: Some(10),
            ..Default::default()
        };
        let query = build_at(&spec, &TRACKS, NOW).unwrap();
        assert!(!query.sql.contains("WHERE"));
        assert!(query.sql.ends_with("LIMIT 10"));
        assert!(query.params.is_empty());
    }

    #[test]
    fn random_sorting_shuffles() {
        let spec = QuerySpec {
            sort_by: Some("random".into()),
            ..Default::default()
        };
        assert!(build_at(&spec, &TRACKS, NOW)
            .unwrap()
            .sql
            .contains("ORDER BY RANDOM()"));
    }

    #[test]
    fn rejects_an_unknown_sort_field() {
        let spec = QuerySpec {
            sort_by: Some("loudness".into()),
            ..Default::default()
        };
        let e = build_at(&spec, &TRACKS, NOW).unwrap_err();
        assert!(e.message.contains("cannot sort by 'loudness'"));
    }

    /// A limit of zero means "no limit", not "no rows" — a form that starts at
    /// 0 must not produce an empty playlist.
    #[test]
    fn a_zero_limit_is_no_limit() {
        let spec = QuerySpec {
            limit: Some(0),
            ..Default::default()
        };
        assert!(!build_at(&spec, &TRACKS, NOW).unwrap().sql.contains("LIMIT"));
    }

    #[test]
    fn validates_against_the_right_schema() {
        assert!(validate("year>2000", &TRACKS).is_ok());
        assert!(validate("", &TRACKS).is_ok());
        // `year` exists on albums, `playcount` does not.
        assert!(validate("year>2000", &ALBUMS).is_ok());
        assert!(validate("playcount>1", &ALBUMS).is_err());
    }

    #[test]
    fn every_schema_round_trips_a_filter_on_each_of_its_fields() {
        for schema in schema::ALL {
            for field in schema.fields {
                let value = match field.kind {
                    FieldKind::Text => "x",
                    FieldKind::Integer => "1",
                    FieldKind::Boolean => "true",
                    FieldKind::Timestamp => "30d",
                };
                let filter = format!("{}=={value}", field.name);
                compile_at(&filter, schema, NOW)
                    .unwrap_or_else(|e| panic!("{}.{}: {e}", schema.name, field.name));
            }
        }
    }
}
