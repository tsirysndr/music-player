//! MCP over stdio: newline-delimited JSON-RPC 2.0.
//!
//! The wire types come from `jsonrpsee-types` rather than being written out
//! again here — ids, error objects and the reserved error codes are exactly the
//! parts of JSON-RPC worth getting from a crate that already has them right.
//!
//! Incoming messages are read into a local struct rather than
//! `jsonrpsee_types::Request`, for one reason: that type requires an id, and
//! MCP's notifications have none. Every client sends
//! `notifications/initialized` immediately after the handshake, so a server
//! that could not parse an id-less message would fail on the first thing it was
//! told.

use anyhow::Error;
use jsonrpsee_types::{ErrorCode, ErrorObject, ErrorObjectOwned, Id, Response, ResponsePayload};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::{session::Session, tools};

/// The revision of MCP this speaks.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// Older revisions that are close enough to serve unchanged. A host pinned to
/// one of these is answered in its own version rather than being told to
/// upgrade, since nothing this server uses differs between them.
const ALSO_SUPPORTED: [&str; 2] = ["2025-03-26", "2024-11-05"];

/// A message from the host. `id` is absent for notifications, which are not to
/// be answered — replying to one is a protocol violation, not a harmless extra.
///
/// The id is kept as raw JSON and converted rather than deserialized as an
/// `Id`: that type borrows from the input, and the input here is one line that
/// is gone by the time the response is written.
#[derive(Deserialize)]
struct Incoming {
    #[serde(default)]
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Option<Value>,
}

/// A JSON-RPC id is a string, a number, or null. Anything else is not one, and
/// answering it under a null id is what the spec asks for.
fn to_id(value: Value) -> Id<'static> {
    match value {
        Value::String(text) => Id::Str(text.into()),
        Value::Number(number) => number.as_u64().map(Id::Number).unwrap_or(Id::Null),
        _ => Id::Null,
    }
}

/// Serve MCP on stdin/stdout until the host closes the pipe.
///
/// Requests are handled one at a time. The protocol permits overlapping them,
/// but every call here is a short local RPC, and a single ordered loop means
/// there is no way for two tool calls to interleave into a queue and leave it
/// in an order the agent did not ask for.
pub async fn serve(mut session: Session) -> Result<(), Error> {
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    let mut stdout = tokio::io::stdout();

    while let Some(line) = lines.next_line().await? {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let message: Incoming = match serde_json::from_str(line) {
            Ok(message) => message,
            Err(cause) => {
                tracing::warn!(%cause, "could not parse a message");
                // Unparseable, so there is no id to answer under, and the spec
                // says to use a null one.
                let error =
                    ErrorObject::owned(ErrorCode::ParseError.code(), cause.to_string(), None::<()>);
                write(&mut stdout, failure(Id::Null, error)).await?;
                continue;
            }
        };

        let Some(id) = message.id.map(to_id) else {
            tracing::debug!(method = %message.method, "notification");
            continue;
        };

        let response = match dispatch(&mut session, &message.method, message.params).await {
            Ok(result) => success(id, result),
            Err(error) => failure(id, error),
        };
        write(&mut stdout, response).await?;
    }

    Ok(())
}

async fn dispatch(
    session: &mut Session,
    method: &str,
    params: Option<Value>,
) -> Result<Value, ErrorObjectOwned> {
    match method {
        "initialize" => Ok(initialize(params)),
        // A liveness check with nothing to report.
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools::catalogue() })),
        "tools/call" => tools::call(session, params).await,
        // Empty lists rather than "method not found": a host that asks is
        // better told there are none than shown an error it must special-case.
        "resources/list" => Ok(json!({ "resources": [] })),
        "prompts/list" => Ok(json!({ "prompts": [] })),
        _ => Err(ErrorObject::owned(
            ErrorCode::MethodNotFound.code(),
            format!("unknown method: {method}"),
            None::<()>,
        )),
    }
}

/// Answer the handshake.
///
/// The host's version is echoed back when it is one this server can speak, so a
/// host pinned to an older revision keeps working; otherwise the newest is
/// offered and the host decides whether it can live with it.
fn initialize(params: Option<Value>) -> Value {
    let requested = params
        .as_ref()
        .and_then(|params| params.get("protocolVersion"))
        .and_then(Value::as_str);

    let version = match requested {
        Some(version) if version == PROTOCOL_VERSION || ALSO_SUPPORTED.contains(&version) => {
            version
        }
        _ => PROTOCOL_VERSION,
    };

    json!({
        "protocolVersion": version,
        "capabilities": { "tools": { "listChanged": false } },
        "serverInfo": {
            "name": "music-player",
            "version": env!("CARGO_PKG_VERSION"),
        },
        // Shown to the model by hosts that surface it. It says the thing that
        // is not guessable from the tool list: this drives the user's real,
        // currently-audible playback.
        "instructions": tools::INSTRUCTIONS,
    })
}

fn success(id: Id<'static>, result: Value) -> Response<'static, Value> {
    Response::new(ResponsePayload::success(result), id)
}

fn failure(id: Id<'static>, error: ErrorObjectOwned) -> Response<'static, Value> {
    Response::new(ResponsePayload::error(error), id)
}

async fn write<W: AsyncWriteExt + Unpin>(
    out: &mut W,
    response: Response<'static, Value>,
) -> Result<(), Error> {
    let mut line = serde_json::to_vec(&response)?;
    line.push(b'\n');
    out.write_all(&line).await?;
    // Unbuffered on purpose: the host is blocked waiting on this line, so
    // holding it back to fill a buffer would stall the agent indefinitely.
    out.flush().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A notification has no id, so it must parse — this is the message every
    /// host sends first.
    #[test]
    fn a_notification_parses_and_carries_no_id() {
        let message: Incoming =
            serde_json::from_str(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
                .expect("a notification is a valid message");
        assert!(message.id.is_none());
        assert_eq!(message.method, "notifications/initialized");
    }

    #[test]
    fn a_request_keeps_its_id_and_params() {
        let message: Incoming = serde_json::from_str(
            r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"now_playing"}}"#,
        )
        .unwrap();
        assert_eq!(message.id.map(to_id), Some(Id::Number(7)));
        assert_eq!(message.params.unwrap()["name"], "now_playing");
    }

    /// String ids are as legal as numeric ones, and must come back unchanged.
    #[test]
    fn a_string_id_survives_the_round_trip() {
        let message: Incoming =
            serde_json::from_str(r#"{"jsonrpc":"2.0","id":"abc","method":"ping"}"#).unwrap();
        let response = success(to_id(message.id.unwrap()), json!({}));
        let encoded = serde_json::to_value(&response).unwrap();
        assert_eq!(encoded["id"], "abc");
        assert_eq!(encoded["jsonrpc"], "2.0");
        assert_eq!(encoded["result"], json!({}));
    }

    #[test]
    fn an_error_response_carries_a_code_and_no_result() {
        let error = ErrorObject::owned(ErrorCode::MethodNotFound.code(), "nope", None::<()>);
        let encoded = serde_json::to_value(failure(Id::Number(1), error)).unwrap();
        assert_eq!(encoded["error"]["code"], -32601);
        assert!(encoded.get("result").is_none());
    }

    /// A host pinned to an older revision is answered in its own version, not
    /// pushed to this one.
    #[test]
    fn the_handshake_echoes_a_version_it_can_speak() {
        let result = initialize(Some(json!({ "protocolVersion": "2024-11-05" })));
        assert_eq!(result["protocolVersion"], "2024-11-05");

        let result = initialize(Some(json!({ "protocolVersion": "1999-01-01" })));
        assert_eq!(result["protocolVersion"], PROTOCOL_VERSION);

        // No version offered at all is not a reason to fail the handshake.
        assert_eq!(initialize(None)["protocolVersion"], PROTOCOL_VERSION);
    }

    #[test]
    fn the_handshake_advertises_tools() {
        let result = initialize(None);
        assert!(result["capabilities"]["tools"].is_object());
        assert_eq!(result["serverInfo"]["name"], "music-player");
    }
}
