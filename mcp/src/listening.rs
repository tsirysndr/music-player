//! MCP tools over the listening-history analytics.
//!
//! These read the DuckDB analytics database directly rather than going through
//! the daemon's gRPC API: the file is local, the questions are read-only, and
//! doing it this way means an agent can answer "what was I listening to last
//! summer" with no daemon running at all.
//!
//! Opened read-only for the same reason it is safe to do that — DuckDB allows
//! many readers or one writer, so a report never blocks an import and two
//! agents can read at once.

use anyhow::{Error, Result};
use music_player_analytics::query::{self, Scope, TopKind, Window};
use music_player_analytics::Analytics;
use serde_json::{json, Value};

/// Tool schemas, appended to the server's list.
pub fn tools() -> Vec<Value> {
    let window = json!({
        "days": {
            "type": "integer", "minimum": 1,
            "description": "Only the last N days. Omit for all of recorded history, \
                            which may span years if an export has been imported."
        },
        "limit": { "type": "integer", "minimum": 1, "maximum": 200,
                   "description": "Rows to return." },
        "local_only": {
            "type": "boolean",
            "description": "Count only tracks the user's local library has, matched on \
                            title and artist. Use when they ask about music they own \
                            rather than everything they have ever played."
        }
    });

    vec![
        tool(
            "listening_overview",
            "Headline listening statistics: how much was played, over what period, \
             how many distinct tracks and artists, session count and longest daily \
             streak. Start here when asked anything general about listening habits.",
            json!({ "type": "object", "properties": window }),
        ),
        tool(
            "listening_top",
            "Most-played artists, tracks, albums or genres over a period.",
            json!({
                "type": "object",
                "properties": {
                    "what": { "type": "string",
                              "enum": ["artists", "tracks", "albums", "genres"],
                              "description": "Defaults to artists." },
                    "days": window["days"].clone(),
                    "limit": window["limit"].clone(),
                    "local_only": window["local_only"].clone()
                }
            }),
        ),
        tool(
            "listening_clock",
            "When listening happens, as counts per weekday and hour of the day, \
             with the average tempo and mood at each hour. Use for questions about \
             routine — mornings, evenings, weekends.",
            json!({ "type": "object", "properties": window }),
        ),
        tool(
            "listening_sessions",
            "How listening sessions behave — how many, how long, how many tracks — \
             and which tracks most often open one. A session is a run of listens \
             with no gap longer than thirty minutes.",
            json!({ "type": "object", "properties": window }),
        ),
        tool(
            "listening_skips",
            "Tracks that get abandoned, with how often and how far in. Only covers \
             sources that record play duration (this player and a Spotify import); \
             a scrobble is submitted after the fact and says nothing about skips.",
            json!({
                "type": "object",
                "properties": {
                    "days": window["days"].clone(),
                    "limit": window["limit"].clone(),
                    "local_only": window["local_only"].clone(),
                    "min_listens": { "type": "integer", "minimum": 1,
                                     "description": "Ignore tracks played fewer times. Defaults to 3." }
                }
            }),
        ),
        tool(
            "listening_drift",
            "How taste moved over time: per period, the average tempo, mood and \
             energy, the median release year of what was played, and the share of \
             listens that were of something never heard before.",
            json!({
                "type": "object",
                "properties": {
                    "bucket": { "type": "string",
                                "enum": ["day", "week", "month", "quarter", "year"],
                                "description": "Defaults to month." },
                    "days": window["days"].clone(),
                    "local_only": window["local_only"].clone()
                }
            }),
        ),
        tool(
            "listening_transitions",
            "What actually follows what, counted within sessions. This is the \
             observed transition graph — useful for building a set that matches \
             how this listener really sequences music.",
            json!({ "type": "object", "properties": window }),
        ),
        tool(
            "listening_rotation",
            "How concentrated listening is (what share the top tracks take, and a \
             Gini coefficient), how much of the local library has ever been played, \
             and tracks once played often but untouched for months.",
            json!({ "type": "object", "properties": window }),
        ),
        tool(
            "listening_on_this_day",
            "What was played on today's date in previous years.",
            json!({ "type": "object",
                    "properties": { "limit": window["limit"].clone() } }),
        ),
    ]
}

pub fn names() -> &'static [&'static str] {
    &[
        "listening_overview",
        "listening_top",
        "listening_clock",
        "listening_sessions",
        "listening_skips",
        "listening_drift",
        "listening_transitions",
        "listening_rotation",
        "listening_on_this_day",
    ]
}

pub async fn call(name: &str, args: &Value) -> Result<Value> {
    // DuckDB is synchronous and these queries are CPU-bound; running one on
    // the async executor would block every other MCP request for its
    // duration. They are milliseconds on a normal history, but a decade of
    // imported scrobbles is not a normal history.
    let name = name.to_owned();
    let args = args.clone();
    tokio::task::spawn_blocking(move || run(&name, &args)).await?
}

fn run(name: &str, args: &Value) -> Result<Value> {
    let analytics = Analytics::open_default_read_only().map_err(|cause| {
        Error::msg(format!(
            "no listening analytics available ({cause}). Run `music-player analytics sync`, \
             or import a history with `music-player analytics import <path>`."
        ))
    })?;

    let scope = Scope {
        window: match args.get("days").and_then(Value::as_i64) {
            Some(days) => Window::last_days(days),
            None => Window::all(),
        },
        local_only: args
            .get("local_only")
            .and_then(Value::as_bool)
            .unwrap_or(false),
    };
    let limit = args
        .get("limit")
        .and_then(Value::as_u64)
        .unwrap_or(20)
        .clamp(1, 200) as u32;

    let value = match name {
        "listening_overview" => {
            let overview = query::overview(&analytics, scope)?;
            let mut value = serde_json::to_value(&overview)?;
            // Without this an agent reading "57,212 listens" has no way to
            // know the number spans imported history rather than this
            // player's own log, and will describe it wrongly.
            value["sources"] = serde_json::to_value(query::origins(&analytics)?)?;
            value
        }
        "listening_top" => {
            let kind = match args
                .get("what")
                .and_then(Value::as_str)
                .unwrap_or("artists")
            {
                "tracks" => TopKind::Tracks,
                "albums" => TopKind::Albums,
                "genres" => TopKind::Genres,
                _ => TopKind::Artists,
            };
            serde_json::to_value(query::top(&analytics, kind, scope, limit)?)?
        }
        "listening_clock" => serde_json::to_value(query::clock(&analytics, scope)?)?,
        "listening_sessions" => {
            serde_json::to_value(query::session_stats(&analytics, scope, limit)?)?
        }
        "listening_skips" => {
            let min_listens = args.get("min_listens").and_then(Value::as_i64).unwrap_or(3);
            serde_json::to_value(query::skips(&analytics, scope, limit, min_listens)?)?
        }
        "listening_drift" => {
            let bucket = args
                .get("bucket")
                .and_then(Value::as_str)
                .unwrap_or("month");
            serde_json::to_value(query::drift(&analytics, scope, bucket)?)?
        }
        "listening_transitions" => {
            serde_json::to_value(query::transitions(&analytics, scope, limit)?)?
        }
        "listening_rotation" => serde_json::to_value(query::rotation(&analytics, scope, limit)?)?,
        "listening_on_this_day" => serde_json::to_value(query::on_this_day(&analytics, limit)?)?,
        other => return Err(Error::msg(format!("unknown listening tool: {other}"))),
    };
    Ok(value)
}

fn tool(name: &str, description: &str, schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": schema })
}
