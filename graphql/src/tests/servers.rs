//! Saved servers, and the two shapes of "add".
//!
//! The interesting case is a backend whose url is not the user's to give:
//! Rocksky's form has no url field, so a client sends an empty one, and
//! demanding one here is what made it unusable.

use super::setup_schema;

/// Every kind the daemon can talk to, with what a form needs to render it.
#[tokio::test]
async fn source_kinds_describe_themselves() {
    let (schema, _, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              query Kinds {
                sourceKinds { kind displayName needsCredentials defaultPort fixedUrl }
              }
            "#,
        )
        .await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);

    let data = resp.data.into_json().unwrap();
    let kinds = data["sourceKinds"].as_array().unwrap();
    let names: Vec<&str> = kinds
        .iter()
        .map(|kind| kind["kind"].as_str().unwrap())
        .collect();
    for expected in [
        "subsonic",
        "jellyfin",
        "music-player",
        "kodi",
        "plex",
        "rocksky",
    ] {
        assert!(
            names.contains(&expected),
            "{expected} is missing: {names:?}"
        );
    }

    let rocksky = kinds
        .iter()
        .find(|kind| kind["kind"] == "rocksky")
        .expect("rocksky is offered");
    assert_eq!(rocksky["fixedUrl"], "https://navidrome.rocksky.app");
    // It is a login, not an address, that the user supplies.
    assert_eq!(rocksky["needsCredentials"], true);

    // Everything else takes a url, so the field stays.
    let subsonic = kinds
        .iter()
        .find(|kind| kind["kind"] == "subsonic")
        .unwrap();
    assert!(subsonic["fixedUrl"].is_null());
}

/// The regression: a client with no url field sends an empty url, and the
/// daemon has to supply the one the factory knows about.
#[tokio::test]
async fn a_fixed_url_backend_saves_without_one() {
    let (schema, _, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: {
                  kind: "rocksky",
                  name: "Rocksky",
                  url: "",
                  username: "tsiry",
                  password: "hunter2"
                }) { id kind name url hasPassword }
              }
            "#,
        )
        .await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);

    let data = resp.data.into_json().unwrap();
    let server = &data["addServer"];
    assert_eq!(server["kind"], "rocksky");
    assert_eq!(server["url"], "https://navidrome.rocksky.app");
    // Stored, never returned.
    assert_eq!(server["hasPassword"], true);
}

/// A url that *is* the user's to give is still required.
#[tokio::test]
async fn a_normal_backend_still_needs_a_url() {
    let (schema, _, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: { kind: "subsonic", name: "NAS", url: "" }) { id }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
    assert!(resp.errors[0].message.contains("needs a url"));
}

/// An unknown kind is refused rather than saved as something unusable.
#[tokio::test]
async fn an_unknown_kind_is_refused() {
    let (schema, _, _, _) = setup_schema().await;
    let resp = schema
        .execute(
            r#"
              mutation Add {
                addServer(input: { kind: "napster", name: "x", url: "http://x" }) { id }
              }
            "#,
        )
        .await;
    assert_eq!(resp.errors.len(), 1);
    assert!(resp.errors[0].message.contains("unknown kind"));
}

/// Nothing connected means the local library, not an error.
#[tokio::test]
async fn nothing_is_connected_by_default() {
    let (schema, _, _, _) = setup_schema().await;
    let resp = schema.execute(r#"query { connectedServer { id } }"#).await;
    assert!(resp.errors.is_empty(), "{:?}", resp.errors);
    assert!(resp.data.into_json().unwrap()["connectedServer"].is_null());
}
