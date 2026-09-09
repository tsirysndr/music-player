//! The tools an agent gets, and what they do.
//!
//! Chosen to be enough to DJ with and no more. Every extra tool is context the
//! model pays for on every single turn, so related transport actions share one
//! tool with an `action` argument rather than being six of them, and anything
//! an agent cannot act on — scanning, settings, extensions — is left out.
//!
//! The unit of exchange is a **track id**. Searching and browsing hand back
//! ids, and the queueing tools take them, so an agent builds a set by passing
//! ids around and never has to construct a track or know what a uri is.

use anyhow::Error;
use jsonrpsee_types::{ErrorCode, ErrorObject, ErrorObjectOwned};
use music_player_server::api::metadata::v1alpha1::{Album, Artist, Track};
use music_player_types::types;
use serde_json::{json, Map, Value};

use crate::session::Session;

/// Told to the model at the handshake.
pub const INSTRUCTIONS: &str = "\
Controls the user's music-player daemon: a real audio player on their machine, \
playing out loud right now. Calls take effect immediately and are heard.

To play something: find it with `search` or `browse_library`, then pass the ids \
to `play_tracks` (replaces the queue and starts) or `queue_tracks` (adds without \
interrupting). `now_playing` says what is on and where it is up to.

When acting as a DJ, prefer `queue_tracks` — it builds the set ahead without \
cutting off what the user is listening to. Use `play_tracks` only when they ask \
for something now. Check `now_playing` before changing the queue, and say what \
you queued rather than only that you queued something.";

/// How many rows a listing returns when the caller does not say. Small on
/// purpose: an agent that wants more can page, whereas one handed nine hundred
/// tracks has spent its context before it has chosen anything.
const DEFAULT_LIMIT: i32 = 20;
const MAX_LIMIT: i32 = 100;

/// The tool descriptors, verbatim as `tools/list` returns them.
pub fn catalogue() -> Value {
    json!([
        tool(
            "now_playing",
            "What is playing right now: track, position, whether it is paused, \
             the volume, and how much is queued after it.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "playback_control",
            "Run the transport: resume, pause, skip, go back, or stop.",
            json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["play", "pause", "next", "previous", "stop"],
                        "description": "`play` resumes what is loaded; it does not start \
                                        anything new. `stop` ends playback and leaves the queue."
                    }
                },
                "required": ["action"],
            }),
        ),
        tool(
            "seek",
            "Jump to a position in the current track.",
            json!({
                "type": "object",
                "properties": {
                    "position_seconds": {
                        "type": "number",
                        "minimum": 0,
                        "description": "Seconds from the start of the track."
                    }
                },
                "required": ["position_seconds"],
            }),
        ),
        tool(
            "set_volume",
            "Set the output volume.",
            json!({
                "type": "object",
                "properties": {
                    "volume": {
                        "type": "integer", "minimum": 0, "maximum": 100,
                        "description": "Percent. 0 is silent, not muted."
                    }
                },
                "required": ["volume"],
            }),
        ),
        tool(
            "search",
            "Search the connected library for tracks, albums and artists at once. \
             The first step for almost anything: it returns the ids the queueing \
             tools take.",
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "description": "Free text — a title, an artist, a few words of either." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_LIMIT,
                               "description": "Rows per kind. Defaults to 20." }
                },
                "required": ["query"],
            }),
        ),
        tool(
            "browse_library",
            "Page through the library when there is nothing specific to search for \
             — picking at random, or seeing what is there.",
            json!({
                "type": "object",
                "properties": {
                    "type": { "type": "string", "enum": ["albums", "artists", "tracks"] },
                    "filter": { "type": "string", "description": "Optional substring to narrow by." },
                    "offset": { "type": "integer", "minimum": 0, "description": "Rows to skip, for paging. Defaults to 0." },
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_LIMIT, "description": "Defaults to 20." }
                },
                "required": ["type"],
            }),
        ),
        tool(
            "get_album",
            "An album and its track listing, in order.",
            json!({
                "type": "object",
                "properties": { "album_id": { "type": "string" } },
                "required": ["album_id"],
            }),
        ),
        tool(
            "get_artist",
            "An artist, their albums, and their tracks.",
            json!({
                "type": "object",
                "properties": { "artist_id": { "type": "string" } },
                "required": ["artist_id"],
            }),
        ),
        tool(
            "list_playlists",
            "The user's playlists.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "get_playlist",
            "The tracks in a playlist, in order.",
            json!({
                "type": "object",
                "properties": { "playlist_id": { "type": "string" } },
                "required": ["playlist_id"],
            }),
        ),
        tool(
            "play_tracks",
            "Replace the queue with these tracks and start playing. Interrupts \
             whatever is on — use `queue_tracks` unless the user asked to hear \
             something now.",
            json!({
                "type": "object",
                "properties": {
                    "track_ids": {
                        "type": "array", "items": { "type": "string" }, "minItems": 1,
                        "description": "Played in the order given."
                    },
                    "start_index": {
                        "type": "integer", "minimum": 0,
                        "description": "Which of them to start on. Defaults to the first."
                    }
                },
                "required": ["track_ids"],
            }),
        ),
        tool(
            "queue_tracks",
            "Add tracks to the queue without disturbing what is playing. The DJ's \
             tool: build the set ahead of the listener.",
            json!({
                "type": "object",
                "properties": {
                    "track_ids": { "type": "array", "items": { "type": "string" }, "minItems": 1 },
                    "position": {
                        "type": "string", "enum": ["next", "end"],
                        "description": "`next` puts them straight after the current track; \
                                        `end` appends. Defaults to `end`."
                    }
                },
                "required": ["track_ids"],
            }),
        ),
        tool(
            "view_queue",
            "The queue: what has played and what is coming.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "clear_queue",
            "Empty the queue and stop playback.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "set_playback_mode",
            "Turn shuffle and repeat on or off.",
            json!({
                "type": "object",
                "properties": {
                    "shuffle": { "type": "boolean", "description": "Reorders only the tracks that have not played yet." },
                    "repeat": { "type": "string", "enum": ["off", "queue", "track"] }
                },
            }),
        ),
        tool(
            "track_analysis",
            "How a track actually sounds: tempo, mood as valence and energy, \
             loudness, and how long it really is. The basis for choosing what \
             goes with what.",
            json!({
                "type": "object",
                "properties": {
                    "track_id": { "type": "string" },
                    "analyze_now": {
                        "type": "boolean",
                        "description": "Decode it if it has not been analysed. Takes seconds, \
                                        and needs a download for a remote track. Defaults to false."
                    }
                },
                "required": ["track_id"],
            }),
        ),
        tool(
            "similar_tracks",
            "Tracks that would sound good after this one, closest first — by \
             tempo and mood, not by genre or by what anyone else listened to. \
             Only considers tracks that have been analysed.",
            json!({
                "type": "object",
                "properties": {
                    "track_id": { "type": "string" },
                    "limit": { "type": "integer", "minimum": 1, "maximum": MAX_LIMIT,
                               "description": "Defaults to 20." }
                },
                "required": ["track_id"],
            }),
        ),
        tool(
            "analyze_library",
            "Analyse tracks that have not been analysed yet, in the background. \
             Returns immediately; call again with no arguments to see progress. \
             Nothing that needs analysis — similar_tracks, auto_dj, waveforms — \
             works on a track until this has covered it.",
            json!({
                "type": "object",
                "properties": {
                    "limit": {
                        "type": "integer", "minimum": 1, "maximum": 5000,
                        "description": "How many tracks to work through. Omit to just \
                                        report progress without starting anything."
                    }
                },
            }),
        ),
        tool(
            "auto_dj",
            "The automatic DJ: keeps the queue filled with tracks that follow on \
             from what is playing. Steer it with a target — it leans that way \
             over the next few tracks rather than jumping. Never interrupts what \
             is playing.",
            json!({
                "type": "object",
                "properties": {
                    "enabled": { "type": "boolean", "description": "Omit to read the current state without changing it." },
                    "bpm": { "type": "number", "minimum": 20, "maximum": 300,
                             "description": "Tempo to head towards. Half and double time count as the same tempo." },
                    "energy": { "type": "number", "minimum": 0, "maximum": 1,
                                "description": "0 calm, 1 driving. The strongest steer of the three." },
                    "brightness": { "type": "number", "minimum": -1, "maximum": 1,
                                    "description": "-1 dark or sad, 1 bright or happy." },
                    "clear_target": { "type": "boolean",
                                      "description": "Stop steering and let the set drift from wherever it is." }
                },
            }),
        ),
        tool(
            "list_servers",
            "The saved music servers, and which one the library is being read from. \
             `null` for the connected one means the daemon's own local library.",
            json!({ "type": "object", "properties": {} }),
        ),
        tool(
            "connect_server",
            "Read the library from a different server. Does not interrupt playback \
             — where music comes from and where it comes out are separate.",
            json!({
                "type": "object",
                "properties": { "server_id": { "type": "string", "description": "An id from `list_servers`." } },
                "required": ["server_id"],
            }),
        ),
    ])
}

fn tool(name: &str, description: &str, schema: Value) -> Value {
    json!({ "name": name, "description": description, "inputSchema": schema })
}

/// Run a `tools/call`.
///
/// A tool that *fails* still returns Ok: MCP distinguishes a broken call from a
/// broken protocol, and only the latter is a JSON-RPC error. A daemon that is
/// not running is something the model should see and tell the user about, not
/// something the host swallows as a transport fault.
pub async fn call(session: &mut Session, params: Option<Value>) -> Result<Value, ErrorObjectOwned> {
    let params = params.unwrap_or_else(|| json!({}));
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_params("a tool call needs a `name`"))?
        .to_string();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match run(session, &name, &args).await {
        Ok(result) => Ok(content(result, false)),
        Err(cause) => {
            tracing::warn!(tool = %name, %cause, "tool failed");
            Ok(content(json!(cause.to_string()), true))
        }
    }
}

/// Wrap a result the way MCP expects: text blocks, and a flag saying whether it
/// went wrong. JSON as the text — agents read it reliably, and it stays compact.
fn content(value: Value, is_error: bool) -> Value {
    let text = match value {
        Value::String(text) => text,
        other => serde_json::to_string(&other).unwrap_or_else(|_| "null".into()),
    };
    json!({ "content": [{ "type": "text", "text": text }], "isError": is_error })
}

fn invalid_params(message: &str) -> ErrorObjectOwned {
    ErrorObject::owned(
        ErrorCode::InvalidParams.code(),
        message.to_string(),
        None::<()>,
    )
}

async fn run(session: &mut Session, name: &str, args: &Value) -> Result<Value, Error> {
    match name {
        "now_playing" => now_playing(session).await,
        "playback_control" => playback_control(session, args).await,
        "seek" => seek(session, args).await,
        "set_volume" => set_volume(session, args).await,
        "search" => search(session, args).await,
        "browse_library" => browse_library(session, args).await,
        "get_album" => get_album(session, args).await,
        "get_artist" => get_artist(session, args).await,
        "list_playlists" => list_playlists(session).await,
        "get_playlist" => get_playlist(session, args).await,
        "play_tracks" => play_tracks(session, args).await,
        "queue_tracks" => queue_tracks(session, args).await,
        "view_queue" => view_queue(session).await,
        "clear_queue" => clear_queue(session).await,
        "set_playback_mode" => set_playback_mode(session, args).await,
        "track_analysis" => track_analysis(session, args).await,
        "similar_tracks" => similar_tracks(session, args).await,
        "analyze_library" => analyze_library(session, args).await,
        "auto_dj" => auto_dj(session, args).await,
        "list_servers" => list_servers(session).await,
        "connect_server" => connect_server(session, args).await,
        _ => Err(Error::msg(format!("unknown tool: {name}"))),
    }
}

// ---------------------------------------------------------------- playback

async fn now_playing(session: &mut Session) -> Result<Value, Error> {
    let (track, index, position_ms, is_playing) = session.playback().await?.current().await?;
    let volume = session.playback().await?.get_volume().await?;
    let (previous, next) = session.tracklist().await?.list().await?;

    let Some(track) = track else {
        return Ok(json!({
            "playing": false,
            "track": Value::Null,
            "volume": volume,
            "queue_length": previous.len() + next.len(),
            "note": "Nothing is loaded. Use play_tracks to start something.",
        }));
    };

    let duration_ms = (track.duration * 1000.0) as u32;
    Ok(json!({
        "playing": is_playing,
        "track": brief_track(&track),
        "position": clock(position_ms / 1000),
        "position_seconds": position_ms / 1000,
        "duration": clock(duration_ms / 1000),
        "queue_index": index,
        "tracks_after": next.len(),
        "volume": volume,
    }))
}

async fn playback_control(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let action = string(args, "action")?;
    let playback = session.playback().await?;
    match action.as_str() {
        "play" => playback.play().await?,
        "pause" => playback.pause().await?,
        "next" => playback.next().await?,
        "previous" => playback.prev().await?,
        "stop" => playback.stop().await?,
        other => return Err(Error::msg(format!("unknown action: {other}"))),
    }
    // What is playing after a skip is the useful answer, not "ok".
    now_playing(session).await
}

async fn seek(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let seconds = args
        .get("position_seconds")
        .and_then(Value::as_f64)
        .ok_or_else(|| Error::msg("`position_seconds` is required"))?;
    if seconds < 0.0 {
        return Err(Error::msg("`position_seconds` cannot be negative"));
    }
    session
        .playback()
        .await?
        .seek((seconds * 1000.0) as u32)
        .await?;
    Ok(json!({ "ok": true, "position": clock(seconds as u32) }))
}

async fn set_volume(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let volume = args
        .get("volume")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::msg("`volume` is required"))?;
    if volume > 100 {
        return Err(Error::msg("`volume` is a percentage, so at most 100"));
    }
    session.playback().await?.set_volume(volume as u32).await?;
    Ok(json!({ "ok": true, "volume": volume }))
}

// ----------------------------------------------------------------- library

async fn search(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let query = string(args, "query")?;
    let limit = limit_of(args) as usize;
    let results = session.library().await?.search(&query).await?;

    Ok(json!({
        "tracks": results.tracks.iter().take(limit).map(brief_track).collect::<Vec<_>>(),
        "albums": results.albums.iter().take(limit).map(brief_album).collect::<Vec<_>>(),
        "artists": results.artists.iter().take(limit).map(brief_artist).collect::<Vec<_>>(),
    }))
}

async fn browse_library(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let kind = string(args, "type")?;
    let filter = args
        .get("filter")
        .and_then(Value::as_str)
        .map(str::to_string);
    let offset = args.get("offset").and_then(Value::as_i64).unwrap_or(0) as i32;
    let limit = limit_of(args);
    let library = session.library().await?;

    let items = match kind.as_str() {
        "albums" => library
            .albums(filter, offset, limit)
            .await?
            .iter()
            .map(brief_album)
            .collect::<Vec<_>>(),
        "artists" => library
            .artists(filter, offset, limit)
            .await?
            .iter()
            .map(brief_artist)
            .collect::<Vec<_>>(),
        "tracks" => library
            .songs(filter, offset, limit)
            .await?
            .iter()
            .map(brief_track)
            .collect::<Vec<_>>(),
        other => {
            return Err(Error::msg(format!(
                "`type` must be albums, artists or tracks, not {other}"
            )))
        }
    };

    // The next offset, so paging does not need arithmetic on the agent's side.
    let next_offset = if items.len() as i32 == limit {
        json!(offset + limit)
    } else {
        Value::Null
    };
    Ok(json!({ "type": kind, "items": items, "next_offset": next_offset }))
}

async fn get_album(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "album_id")?;
    let album = session
        .library()
        .await?
        .album(&id)
        .await?
        .ok_or_else(|| Error::msg(format!("no album with id {id}")))?;

    let tracks = album
        .tracks
        .iter()
        .map(|song| {
            json!({
                "id": song.id,
                "title": song.title,
                "artist": song.artist,
                "track_number": song.track_number,
                "duration": clock(song.duration as u32),
            })
        })
        .collect::<Vec<_>>();

    Ok(json!({
        "id": album.id,
        "title": album.title,
        "artist": album.artist,
        "year": album.year,
        "genres": album.genres,
        "tracks": tracks,
    }))
}

async fn get_artist(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "artist_id")?;
    let artist = session
        .library()
        .await?
        .artist(&id)
        .await?
        .ok_or_else(|| Error::msg(format!("no artist with id {id}")))?;

    Ok(json!({
        "id": artist.id,
        "name": artist.name,
        "genres": artist.genres,
        "albums": artist.albums.iter().map(brief_album).collect::<Vec<_>>(),
        "tracks": artist.songs.iter().map(|song| json!({
            "id": song.id,
            "title": song.title,
            "artist": song.artist,
            "duration": clock(song.duration as u32),
        })).collect::<Vec<_>>(),
    }))
}

async fn list_playlists(session: &mut Session) -> Result<Value, Error> {
    let playlists = session.playlist().await?.list_all().await?;
    Ok(json!(playlists
        .iter()
        .map(|playlist| json!({
            "id": playlist.id,
            "name": playlist.name,
            "tracks": playlist.tracks.len(),
        }))
        .collect::<Vec<_>>()))
}

async fn get_playlist(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "playlist_id")?;
    let tracks = session.playlist().await?.list_songs(&id).await?;
    Ok(json!({
        "id": id,
        "tracks": tracks.iter().map(brief_typed_track).collect::<Vec<_>>(),
    }))
}

// ------------------------------------------------------------------- queue

async fn play_tracks(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let ids = track_ids(args)?;
    let start = args.get("start_index").and_then(Value::as_i64).unwrap_or(0);
    if start < 0 || start as usize >= ids.len() {
        return Err(Error::msg(format!(
            "`start_index` must be between 0 and {}",
            ids.len() - 1
        )));
    }

    let tracks = resolve(session, &ids).await?;
    let starting = tracks[start as usize].clone();
    session
        .tracklist()
        .await?
        .load_tracks(tracks, start as i32)
        .await?;

    Ok(json!({
        "ok": true,
        "queued": ids.len(),
        "now_playing": brief_typed_track(&starting),
    }))
}

async fn queue_tracks(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let ids = track_ids(args)?;
    let position = args
        .get("position")
        .and_then(Value::as_str)
        .unwrap_or("end")
        .to_string();
    let tracks = resolve(session, &ids).await?;
    let titles = tracks.iter().map(brief_typed_track).collect::<Vec<_>>();
    let tracklist = session.tracklist().await?;

    match position.as_str() {
        "end" => tracklist.add_tracks(tracks).await?,
        "next" => {
            // Each one lands directly after the current track, so inserting
            // them front-to-back would play them back-to-front. Reversed, they
            // come out in the order the caller asked for.
            for track in tracks.into_iter().rev() {
                tracklist.play_next(track).await?;
            }
        }
        other => {
            return Err(Error::msg(format!(
                "`position` must be `next` or `end`, not {other}"
            )))
        }
    }

    Ok(json!({ "ok": true, "position": position, "queued": titles }))
}

async fn view_queue(session: &mut Session) -> Result<Value, Error> {
    let (previous, next) = session.tracklist().await?.list().await?;
    Ok(json!({
        "played": previous.iter().map(brief_track).collect::<Vec<_>>(),
        "upcoming": next.iter().map(brief_track).collect::<Vec<_>>(),
    }))
}

async fn clear_queue(session: &mut Session) -> Result<Value, Error> {
    session.tracklist().await?.clear().await?;
    Ok(json!({ "ok": true }))
}

async fn set_playback_mode(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let shuffle = args.get("shuffle").and_then(Value::as_bool);
    let repeat = args.get("repeat").and_then(Value::as_str);
    if shuffle.is_none() && repeat.is_none() {
        return Err(Error::msg("set `shuffle`, `repeat`, or both"));
    }

    let tracklist = session.tracklist().await?;
    if let Some(shuffle) = shuffle {
        tracklist.shuffle(shuffle).await?;
    }
    if let Some(repeat) = repeat {
        let mode = match repeat {
            "off" => 0,
            "queue" => 1,
            "track" => 2,
            other => {
                return Err(Error::msg(format!(
                    "`repeat` must be off, queue or track, not {other}"
                )))
            }
        };
        tracklist.set_repeat(mode).await?;
    }

    Ok(json!({ "ok": true, "shuffle": shuffle, "repeat": repeat }))
}

// ---------------------------------------------------------------- analysis

async fn track_analysis(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "track_id")?;
    let now = args
        .get("analyze_now")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let Some(analysis) = session.analysis().await?.track(&id, now).await? else {
        return Ok(json!({
            "track_id": id,
            "analyzed": false,
            "note": "Not analysed yet. Pass analyze_now, or run analyze_library.",
        }));
    };

    Ok(json!({
        "track_id": id,
        "analyzed": true,
        "bpm": analysis.bpm.map(|bpm| bpm.round()),
        "bpm_confidence": analysis.bpm_confidence,
        // Named rather than passed through: "valence" and "arousal" are terms
        // of art, and the point is for these to be usable without knowing them.
        "energy": analysis.arousal,
        "brightness": analysis.valence,
        "moods": analysis.moods.iter().map(|mood| mood.name.clone()).collect::<Vec<_>>(),
        "loudness_lufs": analysis.lufs,
        "duration": clock(analysis.duration as u32),
    }))
}

async fn similar_tracks(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "track_id")?;
    let limit = limit_of(args);
    let tracks = session.analysis().await?.similar(&id, limit).await?;
    Ok(json!({
        "after": id,
        "tracks": tracks.iter().map(brief_track).collect::<Vec<_>>(),
    }))
}

async fn analyze_library(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let analysis = session.analysis().await?;

    // No limit means "how is it going" — asking for progress should not start
    // work as a side effect.
    if let Some(limit) = args.get("limit").and_then(Value::as_i64) {
        let queued = analysis
            .analyze_library(limit.clamp(1, 5_000) as i32)
            .await?;
        let status = analysis.status().await?;
        return Ok(json!({
            "started": true,
            "queued": queued,
            "analyzed_so_far": status.analyzed,
        }));
    }

    let status = analysis.status().await?;
    Ok(json!({
        "analyzed": status.analyzed,
        "running": status.running,
        "remaining": status.remaining,
    }))
}

async fn auto_dj(session: &mut Session, args: &Value) -> Result<Value, Error> {
    use music_player_server::api::music::v1alpha1::AutoDjTarget;

    let analysis = session.analysis().await?;

    let Some(enabled) = args.get("enabled").and_then(Value::as_bool) else {
        // Reading the state must not change it.
        return Ok(auto_dj_state(&analysis.auto_dj().await?));
    };

    let clear = args
        .get("clear_target")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let bpm = args.get("bpm").and_then(Value::as_f64).map(|v| v as f32);
    let energy = args.get("energy").and_then(Value::as_f64).map(|v| v as f32);
    let brightness = args
        .get("brightness")
        .and_then(Value::as_f64)
        .map(|v| v as f32);

    // Absent leaves the target alone, an empty one clears it. Sending a target
    // only when something was actually said is what keeps "turn it on" from
    // silently wiping a steer set a moment ago.
    let target = if clear {
        Some(AutoDjTarget::default())
    } else if bpm.is_some() || energy.is_some() || brightness.is_some() {
        Some(AutoDjTarget {
            bpm,
            arousal: energy,
            valence: brightness,
        })
    } else {
        None
    };

    let state = analysis.set_auto_dj(enabled, target).await?;
    Ok(auto_dj_state(&state))
}

fn auto_dj_state(state: &music_player_server::api::music::v1alpha1::AutoDjState) -> Value {
    let target = state.target.as_ref();
    json!({
        "enabled": state.enabled,
        "target": {
            "bpm": target.and_then(|t| t.bpm),
            "energy": target.and_then(|t| t.arousal),
            "brightness": target.and_then(|t| t.valence),
        },
        "queued_ahead": state.queued_ahead,
        "candidates": state.candidates,
        "note": if state.candidates == 0 {
            "No analysed tracks, so auto-dj has nothing to choose from. Run analyze_library."
        } else {
            ""
        },
    })
}

// ----------------------------------------------------------------- servers

async fn list_servers(session: &mut Session) -> Result<Value, Error> {
    let servers = session.servers().await?.list().await?;
    let connected = session.servers().await?.connected().await?;
    Ok(json!({
        "connected": connected.as_ref().map(|server| json!({
            "id": server.id, "name": server.name, "url": server.url,
        })),
        "servers": servers.iter().map(|server| json!({
            "id": server.id,
            "kind": server.kind,
            "name": server.name,
            "url": server.url,
            "connected": server.connected,
        })).collect::<Vec<_>>(),
    }))
}

async fn connect_server(session: &mut Session, args: &Value) -> Result<Value, Error> {
    let id = string(args, "server_id")?;
    let server = session.servers().await?.connect(&id).await?;
    Ok(
        json!({ "ok": true, "connected": { "id": server.id, "name": server.name, "url": server.url } }),
    )
}

// ----------------------------------------------------------------- helpers

/// Turn ids into whole tracks.
///
/// The queue is loaded with tracks, not ids: the daemon would otherwise have to
/// look each id up in a local table, which holds nothing at all when the
/// library is a remote server. Resolving here goes through the same read path
/// the ids came from, so it works either way.
async fn resolve(session: &mut Session, ids: &[String]) -> Result<Vec<types::Track>, Error> {
    let library = session.library().await?;
    let mut tracks = Vec::with_capacity(ids.len());
    for id in ids {
        let track = library
            .song(id)
            .await?
            .ok_or_else(|| Error::msg(format!("no track with id {id}")))?;
        tracks.push(types::Track::from(track));
    }
    Ok(tracks)
}

fn track_ids(args: &Value) -> Result<Vec<String>, Error> {
    let ids = args
        .get("track_ids")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::msg("`track_ids` is required"))?
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect::<Vec<_>>();
    if ids.is_empty() {
        return Err(Error::msg(
            "`track_ids` is empty — search or browse first to get ids",
        ));
    }
    Ok(ids)
}

fn string(args: &Value, key: &str) -> Result<String, Error> {
    args.get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| Error::msg(format!("`{key}` is required")))
}

fn limit_of(args: &Value) -> i32 {
    args.get("limit")
        .and_then(Value::as_i64)
        .map(|limit| limit.clamp(1, MAX_LIMIT as i64) as i32)
        .unwrap_or(DEFAULT_LIMIT)
}

/// `m:ss`, because "213.4" is a number an agent has to convert before it can
/// say it out loud.
fn clock(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

/// Only the fields worth spending context on. Covers, bios, lyrics and uris are
/// all dropped: an agent picks by title and artist, and plays by id.
fn brief_track(track: &Track) -> Value {
    let mut fields = Map::new();
    fields.insert("id".into(), json!(track.id));
    fields.insert("title".into(), json!(track.title));
    fields.insert("artist".into(), json!(track.artist));
    if let Some(album) = &track.album {
        fields.insert("album".into(), json!(album.title));
    }
    fields.insert("duration".into(), json!(clock(track.duration as u32)));
    Value::Object(fields)
}

fn brief_typed_track(track: &types::Track) -> Value {
    json!({
        "id": track.id,
        "title": track.title,
        "artist": track.artist,
        "album": track.album.as_ref().map(|album| album.title.clone()),
        "duration": clock(track.duration.unwrap_or_default() as u32),
    })
}

fn brief_album(album: &Album) -> Value {
    let mut fields = Map::new();
    fields.insert("id".into(), json!(album.id));
    fields.insert("title".into(), json!(album.title));
    fields.insert("artist".into(), json!(album.artist));
    if album.year > 0 {
        fields.insert("year".into(), json!(album.year));
    }
    Value::Object(fields)
}

fn brief_artist(artist: &Artist) -> Value {
    json!({ "id": artist.id, "name": artist.name })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        catalogue()
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap().to_string())
            .collect()
    }

    /// Every tool the dispatcher knows must be advertised, and every advertised
    /// tool must dispatch. A tool in one list and not the other is either
    /// invisible to the agent or an error when it is called.
    #[test]
    fn the_catalogue_and_the_dispatcher_agree() {
        // Kept as a literal rather than derived, so adding a tool to only one
        // of the two places fails here instead of at runtime.
        let expected = [
            "now_playing",
            "playback_control",
            "seek",
            "set_volume",
            "search",
            "browse_library",
            "get_album",
            "get_artist",
            "list_playlists",
            "get_playlist",
            "play_tracks",
            "queue_tracks",
            "view_queue",
            "clear_queue",
            "set_playback_mode",
            "track_analysis",
            "similar_tracks",
            "analyze_library",
            "auto_dj",
            "list_servers",
            "connect_server",
        ];
        assert_eq!(names(), expected);
    }

    /// Hosts reject a tool with no schema, and models guess badly at one with no
    /// description.
    #[test]
    fn every_tool_is_fully_described() {
        for tool in catalogue().as_array().unwrap() {
            let name = tool["name"].as_str().unwrap();
            assert!(
                tool["description"].as_str().is_some_and(|d| d.len() > 20),
                "{name} needs a description"
            );
            assert_eq!(tool["inputSchema"]["type"], "object", "{name}");
        }
    }

    /// Required arguments must exist as properties, or the model is asked for
    /// something the schema never mentions.
    #[test]
    fn required_arguments_are_declared_properties() {
        for tool in catalogue().as_array().unwrap() {
            let schema = &tool["inputSchema"];
            let Some(required) = schema["required"].as_array() else {
                continue;
            };
            for key in required {
                let key = key.as_str().unwrap();
                assert!(
                    schema["properties"].get(key).is_some(),
                    "{} requires `{key}` but does not declare it",
                    tool["name"]
                );
            }
        }
    }

    #[test]
    fn durations_are_read_as_a_clock() {
        assert_eq!(clock(0), "0:00");
        assert_eq!(clock(9), "0:09");
        assert_eq!(clock(213), "3:33");
        assert_eq!(clock(3600), "60:00");
    }

    /// A listing must never be able to return the whole library in one call.
    #[test]
    fn limits_are_bounded_at_both_ends() {
        assert_eq!(limit_of(&json!({})), DEFAULT_LIMIT);
        assert_eq!(limit_of(&json!({ "limit": 5 })), 5);
        assert_eq!(limit_of(&json!({ "limit": 100_000 })), MAX_LIMIT);
        assert_eq!(limit_of(&json!({ "limit": 0 })), 1);
        // Not a number at all falls back rather than failing the call.
        assert_eq!(limit_of(&json!({ "limit": "lots" })), DEFAULT_LIMIT);
    }

    #[test]
    fn track_ids_must_be_a_non_empty_list_of_strings() {
        assert!(track_ids(&json!({})).is_err());
        assert!(track_ids(&json!({ "track_ids": [] })).is_err());
        // Nulls and numbers are dropped; if nothing usable is left, that is the
        // empty case and an error, not a silent no-op on the queue.
        assert!(track_ids(&json!({ "track_ids": [1, null] })).is_err());
        assert_eq!(
            track_ids(&json!({ "track_ids": ["a", "b"] })).unwrap(),
            vec!["a".to_string(), "b".to_string()]
        );
    }

    #[test]
    fn a_missing_string_argument_says_which_one() {
        let error = string(&json!({}), "query").unwrap_err().to_string();
        assert!(error.contains("query"), "{error}");
        // Whitespace is not an argument.
        assert!(string(&json!({ "query": "  " }), "query").is_err());
    }

    /// A failed tool is a *result* the model can read, not a JSON-RPC error the
    /// host swallows.
    #[test]
    fn a_tool_failure_is_reported_in_band() {
        let result = content(json!("the daemon is not running"), true);
        assert_eq!(result["isError"], true);
        assert_eq!(result["content"][0]["type"], "text");
        assert_eq!(result["content"][0]["text"], "the daemon is not running");
    }

    /// Structured results are serialised into the text block, since that is the
    /// one content type every host renders.
    #[test]
    fn structured_results_are_serialised_as_text() {
        let result = content(json!({ "ok": true }), false);
        assert_eq!(result["isError"], false);
        assert_eq!(result["content"][0]["text"], r#"{"ok":true}"#);
    }

    #[tokio::test]
    async fn an_unknown_tool_is_an_error_the_model_sees() {
        let mut session = Session::new("127.0.0.1".into(), 1);
        let result = call(&mut session, Some(json!({ "name": "make_coffee" })))
            .await
            .unwrap();
        assert_eq!(result["isError"], true);
        assert!(result["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("make_coffee"));
    }

    /// A call with no `name` is malformed protocol, not a failed tool — this is
    /// the one case that is a JSON-RPC error.
    #[tokio::test]
    async fn a_call_without_a_name_is_a_protocol_error() {
        let mut session = Session::new("127.0.0.1".into(), 1);
        let error = call(&mut session, Some(json!({ "arguments": {} })))
            .await
            .unwrap_err();
        assert_eq!(error.code(), ErrorCode::InvalidParams.code());
    }

    /// The daemon being down must read as a failed tool with an actionable
    /// message, not as a transport fault the model never learns about.
    #[tokio::test]
    async fn a_daemon_that_is_not_running_is_explained() {
        // Port 1 is privileged and nothing listens there.
        let mut session = Session::new("127.0.0.1".into(), 1);
        let result = call(&mut session, Some(json!({ "name": "now_playing" })))
            .await
            .unwrap();
        assert_eq!(result["isError"], true);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("music-player"), "{text}");
    }
}
