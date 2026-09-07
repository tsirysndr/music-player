//! Compile a parsed filter into a SQLite `WHERE` clause plus bound values.
//!
//! The output is always `(sql, params)`. No user-supplied text ever reaches
//! `sql` — field names are resolved to fixed column expressions through the
//! [`Schema`], and every literal becomes a `?`.

use crate::ast::{parse, Constraint, Node};
use crate::error::{Error, Result};
use crate::lexer::Comparison;
use crate::schema::{FieldKind, Schema};

/// A value bound to a `?` in the generated SQL.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Text(String),
    Integer(i64),
}

/// A compiled filter: a `WHERE` body and the values its placeholders take.
#[derive(Clone, Debug, PartialEq)]
pub struct Filter {
    pub sql: String,
    pub params: Vec<Value>,
}

/// How many seconds a relative-age suffix stands for.
fn unit_seconds(suffix: char) -> Option<i64> {
    Some(match suffix {
        's' => 1,
        'h' => 3_600,
        'd' => 86_400,
        'w' => 604_800,
        'm' => 2_592_000,  // 30 days
        'y' => 31_536_000, // 365 days
        _ => return None,
    })
}

/// Compile `input` against `schema`.
///
/// `now` is the reference point for relative ages like `30d`, as a unix
/// timestamp — passed in rather than read from the clock so the same filter
/// compiles to the same SQL in a test.
pub fn compile_at(input: &str, schema: &Schema, now: i64) -> Result<Filter> {
    let node = parse(input)?;
    let mut params = Vec::new();
    let sql = emit(&node, schema, now, &mut params)?;
    Ok(Filter { sql, params })
}

/// Compile `input` against `schema`, relative to the current time.
pub fn compile(input: &str, schema: &Schema) -> Result<Filter> {
    compile_at(input, schema, unix_now())
}

fn unix_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

fn emit(node: &Node, schema: &Schema, now: i64, params: &mut Vec<Value>) -> Result<String> {
    match node {
        Node::And(children) => join(children, " AND ", schema, now, params),
        Node::Or(children) => join(children, " OR ", schema, now, params),
        Node::Compare(constraint) => comparison(constraint, schema, now, params),
    }
}

fn join(
    children: &[Node],
    sep: &str,
    schema: &Schema,
    now: i64,
    params: &mut Vec<Value>,
) -> Result<String> {
    let mut parts = Vec::with_capacity(children.len());
    for child in children {
        parts.push(emit(child, schema, now, params)?);
    }
    Ok(format!("({})", parts.join(sep)))
}

fn comparison(
    constraint: &Constraint,
    schema: &Schema,
    now: i64,
    params: &mut Vec<Value>,
) -> Result<String> {
    let field = schema.field(&constraint.field).ok_or_else(|| {
        Error::new(
            format!(
                "unknown field '{}' — try one of: {}",
                constraint.field,
                schema.field_names().join(", ")
            ),
            constraint.at,
        )
    })?;
    let column = field.column;

    match constraint.op {
        Comparison::Null => return Ok(empty_test(column, field.kind, true)),
        Comparison::NotNull => return Ok(empty_test(column, field.kind, false)),
        _ => {}
    }

    if constraint.op.is_list() {
        let mut placeholders = Vec::with_capacity(constraint.values.len());
        for raw in &constraint.values {
            params.push(bind(raw, field.kind, now, constraint)?);
            placeholders.push("?");
        }
        let list = placeholders.join(", ");
        // A NULL column is in no list, and is also in no complement of one;
        // `=out=` has to say so explicitly or NULLs vanish from the result.
        return Ok(match constraint.op {
            Comparison::In => format!("{column} IN ({list})"),
            _ => format!("({column} IS NULL OR {column} NOT IN ({list}))"),
        });
    }

    let raw = &constraint.values[0];

    // `=like=` is a substring match with `*` as the wildcard. It is the one
    // operator whose value shape differs from the column's own type.
    if constraint.op == Comparison::Like {
        let pattern = if raw.contains('*') {
            raw.replace('*', "%")
        } else {
            format!("%{raw}%")
        };
        params.push(Value::Text(pattern));
        return Ok(format!("{column} LIKE ? ESCAPE '\\'"));
    }

    let value = bind(raw, field.kind, now, constraint)?;

    // Text equality is case-insensitive: nobody typing a filter means
    // `artist==radiohead` to miss "Radiohead".
    if field.kind == FieldKind::Text && matches!(constraint.op, Comparison::Eq | Comparison::Ne) {
        params.push(value);
        return Ok(match constraint.op {
            Comparison::Eq => format!("{column} = ? COLLATE NOCASE"),
            _ => format!("({column} IS NULL OR {column} <> ? COLLATE NOCASE)"),
        });
    }

    params.push(value);
    let operator = match constraint.op {
        Comparison::Eq => "=",
        Comparison::Ne => "<>",
        Comparison::Gt => ">",
        Comparison::Ge => ">=",
        Comparison::Lt => "<",
        Comparison::Le => "<=",
        // Handled above.
        Comparison::In | Comparison::Out | Comparison::Like => unreachable!(),
        Comparison::Null | Comparison::NotNull => unreachable!(),
    };
    // `!=` must keep rows whose column is NULL: in SQL `NULL <> 'x'` is NULL,
    // not true, so a track with no genre would drop out of `genre!=rock`.
    if constraint.op == Comparison::Ne {
        return Ok(format!("({column} IS NULL OR {column} <> ?)"));
    }
    Ok(format!("{column} {operator} ?"))
}

/// `=null=` / `=notnull=`. For text an empty string reads as "no value" too —
/// the scanner writes `''` for a missing tag rather than NULL.
fn empty_test(column: &str, kind: FieldKind, want_empty: bool) -> String {
    match (kind, want_empty) {
        (FieldKind::Text, true) => format!("({column} IS NULL OR {column} = '')"),
        (FieldKind::Text, false) => format!("({column} IS NOT NULL AND {column} <> '')"),
        (_, true) => format!("{column} IS NULL"),
        (_, false) => format!("{column} IS NOT NULL"),
    }
}

fn bind(raw: &str, kind: FieldKind, now: i64, constraint: &Constraint) -> Result<Value> {
    match kind {
        FieldKind::Text => Ok(Value::Text(raw.to_owned())),
        FieldKind::Integer => raw.parse::<i64>().map(Value::Integer).map_err(|_| {
            Error::new(
                format!("'{}' expects a number, got '{raw}'", constraint.field),
                constraint.at,
            )
        }),
        FieldKind::Boolean => match raw.to_ascii_lowercase().as_str() {
            "true" | "yes" | "1" => Ok(Value::Integer(1)),
            "false" | "no" | "0" => Ok(Value::Integer(0)),
            _ => Err(Error::new(
                format!("'{}' expects true or false, got '{raw}'", constraint.field),
                constraint.at,
            )),
        },
        FieldKind::Timestamp => timestamp(raw, now, constraint),
    }
}

/// A timestamp value: a relative age (`30d`), a unix timestamp, or an ISO
/// date. Relative ages are what smart playlists are actually written with —
/// "played in the last 30 days" — and they resolve against `now` here rather
/// than being re-evaluated by SQLite, so the SQL stays a plain comparison.
fn timestamp(raw: &str, now: i64, constraint: &Constraint) -> Result<Value> {
    let trimmed = raw.trim();
    if let Some(suffix) = trimmed.chars().last() {
        if let Some(seconds) = unit_seconds(suffix.to_ascii_lowercase()) {
            let count = &trimmed[..trimmed.len() - suffix.len_utf8()];
            if let Ok(count) = count.parse::<i64>() {
                return Ok(Value::Integer(now - count * seconds));
            }
        }
    }
    if let Ok(seconds) = trimmed.parse::<i64>() {
        return Ok(Value::Integer(seconds));
    }
    // `YYYY-MM-DD`, resolved to midnight UTC.
    if let Some(seconds) = iso_date_to_unix(trimmed) {
        return Ok(Value::Integer(seconds));
    }
    Err(Error::new(
        format!(
            "'{}' expects a date (2024-01-31) or an age (30d, 6m, 1y), got '{raw}'",
            constraint.field
        ),
        constraint.at,
    ))
}

/// Days since the epoch for `YYYY-MM-DD`, as seconds. Proleptic Gregorian, so
/// it agrees with SQLite's own date handling.
fn iso_date_to_unix(text: &str) -> Option<i64> {
    let mut parts = text.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // Howard Hinnant's days_from_civil.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_adjusted = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_adjusted + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some((era * 146_097 + day_of_era - 719_468) * 86_400)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::TRACKS;

    /// A fixed "now" so relative ages compile to a predictable number.
    const NOW: i64 = 1_700_000_000;

    fn compiled(input: &str) -> Filter {
        compile_at(input, &TRACKS, NOW).unwrap()
    }

    #[test]
    fn compiles_a_simple_comparison() {
        let filter = compiled("year>2000");
        assert_eq!(filter.sql, "track.year > ?");
        assert_eq!(filter.params, vec![Value::Integer(2000)]);
    }

    #[test]
    fn text_equality_ignores_case() {
        let filter = compiled("artist==radiohead");
        assert_eq!(filter.sql, "track.artist = ? COLLATE NOCASE");
        assert_eq!(filter.params, vec![Value::Text("radiohead".into())]);
    }

    #[test]
    fn nests_and_or_with_the_right_precedence() {
        let filter = compiled("genre==rock,year>2000;bitrate>=320");
        assert_eq!(
            filter.sql,
            "(track.genre = ? COLLATE NOCASE OR (track.year > ? AND track.bitrate >= ?))"
        );
    }

    #[test]
    fn compiles_value_lists() {
        let filter = compiled("genre=in=(rock,jazz)");
        assert_eq!(filter.sql, "track.genre IN (?, ?)");
        assert_eq!(
            filter.params,
            vec![Value::Text("rock".into()), Value::Text("jazz".into())]
        );
    }

    /// In SQL `NULL <> 'x'` is NULL, not true. Without the explicit IS NULL
    /// arm, every untagged track silently disappears from a `!=` filter.
    #[test]
    fn negations_keep_rows_with_no_value() {
        assert_eq!(
            compiled("genre!=rock").sql,
            "(track.genre IS NULL OR track.genre <> ? COLLATE NOCASE)"
        );
        assert_eq!(
            compiled("genre=out=(rock)").sql,
            "(track.genre IS NULL OR track.genre NOT IN (?))"
        );
    }

    #[test]
    fn like_wraps_or_honours_wildcards() {
        assert_eq!(
            compiled("title=like=love").params,
            vec![Value::Text("%love%".into())]
        );
        assert_eq!(
            compiled("title=like=love*").params,
            vec![Value::Text("love%".into())]
        );
    }

    #[test]
    fn relative_ages_resolve_against_now() {
        let filter = compiled("lastplayed>30d");
        assert_eq!(filter.sql, "track_stats.last_played > ?");
        assert_eq!(filter.params, vec![Value::Integer(NOW - 30 * 86_400)]);
    }

    #[test]
    fn iso_dates_resolve_to_midnight_utc() {
        assert_eq!(iso_date_to_unix("1970-01-01"), Some(0));
        assert_eq!(iso_date_to_unix("2024-01-31"), Some(1_706_659_200));
        assert_eq!(iso_date_to_unix("nope"), None);
    }

    #[test]
    fn booleans_accept_the_usual_spellings() {
        for input in ["liked==true", "liked==yes", "liked==1"] {
            assert_eq!(compiled(input).params, vec![Value::Integer(1)]);
        }
        assert_eq!(compiled("liked==false").params, vec![Value::Integer(0)]);
    }

    /// A track nobody has played has no `track_stats` row, and must still
    /// match `playcount==0`.
    #[test]
    fn missing_stats_count_as_zero() {
        assert_eq!(
            compiled("playcount==0").sql,
            "COALESCE(track_stats.play_count, 0) = ?"
        );
    }

    #[test]
    fn empty_tests_treat_a_blank_tag_as_absent() {
        assert_eq!(
            compiled("genre=null=").sql,
            "(track.genre IS NULL OR track.genre = '')"
        );
        assert_eq!(
            compiled("lastplayed=null=").sql,
            "track_stats.last_played IS NULL"
        );
    }

    #[test]
    fn rejects_unknown_fields_and_bad_values() {
        let e = compile_at("artistt==X", &TRACKS, NOW).unwrap_err();
        assert!(e.message.contains("unknown field 'artistt'"));
        // The message lists what the user could have meant.
        assert!(e.message.contains("artist"));
        assert_eq!(e.at, 0);

        let e = compile_at("year>nineteen", &TRACKS, NOW).unwrap_err();
        assert!(e.message.contains("expects a number"));
    }

    /// The whole injection story: values are bound, never spliced.
    #[test]
    fn values_never_reach_the_sql() {
        let filter = compiled("artist=='; DROP TABLE track; --'");
        assert_eq!(filter.sql, "track.artist = ? COLLATE NOCASE");
        assert_eq!(
            filter.params,
            vec![Value::Text("; DROP TABLE track; --".into())]
        );
    }
}
