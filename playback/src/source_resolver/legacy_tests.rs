//! SQLite + real WAT through the public account boundary and SourceResolver.
//! CI only: descriptors are inspected and dropped; no HTTP/decoder is opened.
use super::*;
use crate::{
    extension_accounts::AccountError,
    managed::{SeekPolicy, StartPosition, Timeline},
};
use migration::{Migrator, MigratorTrait};
use music_player_extensions::{abi, host::HostContext};
use music_player_storage::extension_source_accounts::{ConfigChange, SourceAccount};
use music_player_types::source::LocalSourceRef;
use serde_json::json;
use std::{collections::BTreeMap, future::Future, path::PathBuf};

const PLUGIN: &str = "resolver.legacy.fixture";
const QUERY: &str = "needle";
const GUEST_URI: &str = "https://guest.invalid/temporary?token=synthetic-browse-secret";
const STREAM_URL: &str = "http://127.0.0.1:1/misleading.mp3";
async fn bounded(future: impl Future<Output = ()>) {
    let _serial = crate::extension_accounts::TEST_LOAD_SERIAL.lock().await;
    tokio::time::timeout(Duration::from_secs(30), future)
        .await
        .expect("legacy resolver scenario watchdog");
}

struct Directory(PathBuf);
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

// All exports use the locked Extism kernel ABI: offsets/lengths i64, bytes i32.
fn emit_json(body: &str) -> String {
    let mut code = format!("(local.set $p (call $alloc (i64.const {})))", body.len());
    for (index, byte) in body.bytes().enumerate() {
        code.push_str(&format!(
            "(call $store (i64.add (local.get $p) (i64.const {index})) (i32.const {byte}))"
        ));
    }
    code.push_str(&format!(
        "(call $output (local.get $p) (i64.const {})) (return (i32.const 0))",
        body.len()
    ));
    code
}

// Match the entire actual guest input, including parent/query and the final
// empty-page offset. An unexpected payload traps instead of returning a page.
fn matcher(name: &str, input: &str) -> String {
    let mut code = format!(
        "(func ${name} (result i32) (if (i64.ne (call $input_length) (i64.const {})) (then (return (i32.const 0))))",
        input.len()
    );
    for (index, byte) in input.bytes().enumerate() {
        code.push_str(&format!("(if (i32.ne (call $input (i64.const {index})) (i32.const {byte})) (then (return (i32.const 0))))"));
    }
    code.push_str("(i32.const 1))");
    code
}

fn list_export(export: &str, body: &str, parents: &[&str]) -> String {
    let mut definitions = String::new();
    let mut function = format!(
        "(func (export \"{export}\") (result i32) (local $p i64) (global.set $lists (i32.add (global.get $lists) (i32.const 1)))"
    );
    for (parent_index, parent) in parents.iter().enumerate() {
        for offset in [0, 1] {
            let name = format!("{export}_{parent_index}_{offset}");
            let request = abi::BrowseRequest {
                parent_id: (*parent).into(),
                query: QUERY.into(),
                offset,
                limit: 1,
            };
            definitions.push_str(&matcher(&name, &serde_json::to_string(&request).unwrap()));
            function.push_str(&format!(
                "(if (call ${name}) (then {}))",
                emit_json(if offset == 0 { body } else { "[]" })
            ));
        }
    }
    function.push_str("unreachable)");
    definitions + &function
}

fn write_plugin(directory: &Directory) {
    let plugin_dir = directory.0.join("plugin");
    std::fs::create_dir_all(&plugin_dir).unwrap();
    let mut wat = String::from(
        r#"(module
          (import "extism:host/env" "alloc" (func $alloc (param i64) (result i64)))
          (import "extism:host/env" "store_u8" (func $store (param i64 i32)))
          (import "extism:host/env" "output_set" (func $output (param i64 i64)))
          (import "extism:host/env" "input_length" (func $input_length (result i64)))
          (import "extism:host/env" "input_load_u8" (func $input (param i64) (result i32)))
          (global $resolves (mut i32) (i32.const 0))
          (global $lists (mut i32) (i32.const 0))
          (func $hex (param $n i32) (result i32)
            (if (result i32) (i32.lt_u (local.get $n) (i32.const 10))
              (then (i32.add (local.get $n) (i32.const 48)))
              (else (i32.add (local.get $n) (i32.const 87)))))
        "#,
    );
    let info = r#"{"name":"Resolver fixture","description":"00:00"}"#;
    let info_code = emit_json(info);
    let output_at = info_code.find("(call $output").unwrap();
    wat.push_str("(func (export \"source_info\") (result i32) (local $p i64)");
    wat.push_str(&info_code[..output_at]);
    let start = info.find("00:00").unwrap();
    for (global, at) in [("resolves", start), ("lists", start + 3)] {
        for (index, shift) in [(at, 4), (at + 1, 0)] {
            wat.push_str(&format!("(call $store (i64.add (local.get $p) (i64.const {index})) (call $hex (i32.and (i32.shr_u (global.get ${global}) (i32.const {shift})) (i32.const 15))))"));
        }
    }
    wat.push_str(&info_code[output_at..]);
    wat.push(')');

    let tracks = json!([{
        "id":"track", "title":"Original title", "artist":"Guest artist",
        "duration":301.5, "trackNumber":0, "uri":GUEST_URI
    }])
    .to_string();
    wat.push_str(&list_export(
        "list_tracks",
        &tracks,
        &["", "album", "artist", "playlist"],
    ));
    wat.push_str(&list_export(
        "list_albums",
        r#"[{"id":"album","title":"Album title"}]"#,
        &[""],
    ));
    wat.push_str(&list_export(
        "list_artists",
        r#"[{"id":"artist","name":"Artist title"}]"#,
        &[""],
    ));
    wat.push_str(&list_export(
        "list_playlists",
        r#"[{"id":"playlist","name":"Playlist title"}]"#,
        &[""],
    ));
    let stream_request = serde_json::to_string(&abi::StreamRequest {
        track_id: "track".into(),
    })
    .unwrap();
    wat.push_str(&matcher("stream_request", &stream_request));
    wat.push_str(
        r#"(func (export "get_stream_url") (result i32) (local $p i64)
      (global.set $resolves (i32.add (global.get $resolves) (i32.const 1)))
      (if (i32.eqz (call $stream_request)) (then unreachable))"#,
    );
    wat.push_str(&emit_json(
        &json!({
            "url":STREAM_URL, "mimeType":"audio/flac",
            "headers":r#"{"Authorization":"Bearer synthetic-resolve-secret"}"#
        })
        .to_string(),
    ));
    wat.push_str("))");
    std::fs::write(plugin_dir.join("plugin.wat"), wat).unwrap();
    std::fs::write(
        plugin_dir.join("plugin.json"),
        json!({
            "id":PLUGIN, "name":"Resolver fixture", "entry":"plugin.wat",
            "capabilities":["source"], "sourceApiVersion":1,
            "permissions":{"config":{"server":"A"}}
        })
        .to_string(),
    )
    .unwrap();
}

struct Fixture {
    accounts: ExtensionAccounts,
    resolver: SourceResolver,
    _directory: Directory,
}
impl Fixture {
    async fn new() -> Self {
        let directory = Directory(
            std::env::temp_dir().join(format!("mp-legacy-resolver-{}", uuid::Uuid::new_v4())),
        );
        write_plugin(&directory);
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let accounts = ExtensionAccounts::new(
            db.clone(),
            vec![directory.0.clone()],
            HostContext::default(),
        );
        let resolver = SourceResolver::new(db, "fixture-device".into(), false)
            .with_extensions(accounts.clone(), vec!["flac".into()]);
        Self {
            accounts,
            resolver,
            _directory: directory,
        }
    }

    async fn create(&self, name: &str) -> SourceAccount {
        // Catalog scans share the process load budget with other CI modules.
        // Create Busy is admission failure before its storage mutation. Never
        // retry a lost reply or any other error; the scenario watchdog bounds us.
        loop {
            match self
                .accounts
                .create(PLUGIN, name, BTreeMap::new(), "t0")
                .await
            {
                Err(AccountError::Busy) => tokio::task::yield_now().await,
                result => return result.unwrap(),
            }
        }
    }

    async fn connect(&self, id: &str) -> SourceAccount {
        // This fixture exclusively owns this account. Admission Busy can retry
        // setup (including a read of source_info); never retry a stream resolve.
        loop {
            match self.accounts.connect(id).await {
                Err(AccountError::Busy) => tokio::task::yield_now().await,
                result => return result.unwrap(),
            }
        }
    }

    async fn page(
        &self,
        row: &SourceAccount,
        list: LegacySourceList,
        parent: Option<&LocalSourceRef>,
        offset: u64,
    ) -> SourceMediaPage {
        self.resolver
            .extension_legacy_media(&row.id, list, parent, Some(QUERY.into()), offset, 1)
            .await
            .unwrap()
    }

    async fn counts(&self, row: &SourceAccount, resolves: u8, lists: u8) {
        let runtime = self.accounts.for_account(&row.id).await.unwrap();
        let info = runtime.legacy_info().await.unwrap();
        assert_eq!(info.description, format!("{resolves:02x}:{lists:02x}"));
    }

    fn reserved(&self, id: &str) -> usize {
        self.accounts
            .status()
            .iter()
            .filter(|s| s.account_id == id)
            .map(|s| s.runtime.reserved)
            .sum()
    }

    async fn released(&self, id: &str) {
        tokio::time::timeout(Duration::from_secs(5), async {
            while self.reserved(id) != 0 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("local legacy lease released");
    }
}

fn local_handle(id: &str, row: &SourceAccount, kind: ResourceKind) -> LocalSourceRef {
    let SourceHandle::Local(source) = SourceHandle::parse(id).unwrap() else {
        panic!("v1 must return a local scope")
    };
    assert_eq!(source.account_id, row.id);
    assert_eq!(source.extension_id(), Some(PLUGIN));
    assert_eq!(
        source.config_revision,
        u64::try_from(row.config_revision).unwrap()
    );
    assert_eq!(source.kind, kind);
    source
}

#[tokio::test]
async fn four_lists_bind_local_handles_and_full_pages_end_with_empty_requests() {
    bounded(async {
        let f = Fixture::new().await;
        let row = f.create("A").await;
        let row = f.connect(&row.id).await;
        assert!(row.remote_server_id.is_none() && row.remote_user_id.is_none());
        let mut track = None;
        let mut parents = Vec::new();
        for (list, kind, title) in [
            (
                LegacySourceList::Tracks,
                ResourceKind::Item,
                "Original title",
            ),
            (LegacySourceList::Albums, ResourceKind::Album, "Album title"),
            (
                LegacySourceList::Artists,
                ResourceKind::Artist,
                "Artist title",
            ),
            (
                LegacySourceList::Playlists,
                ResourceKind::Playlist,
                "Playlist title",
            ),
        ] {
            let page = f.page(&row, list, None, 0).await;
            assert_eq!(page.items.len(), 1);
            assert_eq!(page.next_offset, Some(1));
            let item = &page.items[0];
            assert_eq!(item.title, title);
            let local = local_handle(&item.id, &row, kind);
            if kind == ResourceKind::Item {
                assert!(!item.is_container);
                let value = item.track.as_ref().unwrap();
                assert_eq!(value.id, local.to_handle());
                assert_eq!(value.uri, value.id);
                assert!(!value.uri.contains("synthetic-browse-secret"));
                assert_ne!(value.uri, GUEST_URI);
                assert_eq!(value.artist, "Guest artist");
                assert_eq!(value.duration, Some(301.5));
                assert_eq!(value.track_number, Some(0));
                track = Some(SourceHandle::Local(local));
            } else {
                assert!(item.is_container && item.track.is_none());
                let error = f
                    .resolver
                    .resolve_playback(
                        &SourceHandle::Local(local.clone()),
                        &AudioSelection::Auto,
                        0,
                    )
                    .await
                    .err()
                    .unwrap();
                assert_eq!(error.to_string(), "saved source is not an item");
                parents.push(local);
            }
            let empty = f.page(&row, list, None, page.next_offset.unwrap()).await;
            assert!(empty.items.is_empty());
            assert_eq!(empty.next_offset, None);
        }
        // Each real album/artist/playlist ID is sent as parentId, never its handle.
        for parent in &parents {
            let page = f
                .page(&row, LegacySourceList::Tracks, Some(parent), 0)
                .await;
            assert_eq!(page.items[0].id, track.as_ref().unwrap().to_handle());
            let end = f
                .page(
                    &row,
                    LegacySourceList::Tracks,
                    Some(parent),
                    page.next_offset.unwrap(),
                )
                .await;
            assert!(end.items.is_empty() && end.next_offset.is_none());
        }
        f.counts(&row, 0, 14).await;
        let descriptor = f
            .resolver
            .resolve_playback(&track.unwrap(), &AudioSelection::Auto, 0)
            .await
            .unwrap();
        assert_eq!(descriptor.request.url, STREAM_URL);
        assert_eq!(descriptor.format_ext, "flac"); // MIME, not the .mp3 URL suffix
        assert!(descriptor.request.headers["authorization"] == "Bearer synthetic-resolve-secret");
        assert!(!descriptor.request.follow_redirects);
        assert_eq!(descriptor.start, StartPosition::RequestedOnly);
        assert_eq!(descriptor.requested_offset_ms, 0);
        assert_eq!(descriptor.capabilities.timeline, Timeline::Unknown);
        assert_eq!(descriptor.capabilities.seek_policy, SeekPolicy::None);
        assert!(descriptor.pin.is_none());
        f.counts(&row, 1, 14).await;
        assert_eq!(f.reserved(&row.id), 1);
        drop(descriptor);
        f.released(&row.id).await;
    })
    .await;
}

#[tokio::test]
async fn binding_revision_and_cross_account_parent_guard_the_resolver_and_old_lease() {
    bounded(async {
        let f = Fixture::new().await;
        let unbound = f.create("A").await;
        let unbound_handle = SourceHandle::Local(
            LocalSourceRef::for_extension(
                PLUGIN,
                unbound.id.clone(),
                1,
                ResourceKind::Item,
                "track".into(),
            )
            .unwrap(),
        );
        assert!(f.resolver.validate_handle(&unbound_handle).await.is_err());
        assert!(f
            .resolver
            .resolve_playback(&unbound_handle, &AudioSelection::Auto, 0)
            .await
            .is_err());
        assert!(f
            .resolver
            .extension_legacy_media(
                &unbound.id,
                LegacySourceList::Tracks,
                None,
                Some(QUERY.into()),
                0,
                1
            )
            .await
            .is_err());
        assert!(f.accounts.status().is_empty());
        let a = f.connect(&unbound.id).await;
        let page = f.page(&a, LegacySourceList::Tracks, None, 0).await;
        let old = SourceHandle::parse(&page.items[0].id).unwrap();
        let albums = f.page(&a, LegacySourceList::Albums, None, 0).await;
        let parent = local_handle(&albums.items[0].id, &a, ResourceKind::Album);
        let b = f.create("B").await;
        let b = f.connect(&b.id).await;
        let other_page = f.page(&b, LegacySourceList::Tracks, None, 0).await;
        let other = SourceHandle::parse(&other_page.items[0].id).unwrap();
        assert_eq!(old.item_id(), other.item_id());
        assert_ne!(old.to_handle(), other.to_handle());
        let error = f
            .resolver
            .extension_legacy_media(
                &b.id,
                LegacySourceList::Tracks,
                Some(&parent),
                Some(QUERY.into()),
                0,
                1,
            )
            .await
            .err()
            .unwrap();
        assert_eq!(error.to_string(), "invalid extension browse parent");
        f.counts(&a, 0, 2).await;
        f.counts(&b, 0, 1).await;

        let old_descriptor = f
            .resolver
            .resolve_playback(&old, &AudioSelection::Auto, 0)
            .await
            .unwrap();
        // Update also scans the catalog under the shared load budget; only
        // admission Busy proves this storage mutation has not run.
        let edited = loop {
            match f
                .accounts
                .update(
                    &a,
                    &a.name,
                    BTreeMap::from([("server".into(), ConfigChange::Set("B".into()))]),
                    "t1",
                )
                .await
            {
                Err(AccountError::Busy) => tokio::task::yield_now().await,
                result => break result.unwrap(),
            }
        };
        assert_eq!(edited.config_revision, a.config_revision + 1);
        assert_eq!(edited.local_bound_revision, None);
        assert!(f.resolver.validate_handle(&old).await.is_err());
        assert!(f
            .resolver
            .resolve_playback(&old, &AudioSelection::Auto, 0)
            .await
            .is_err());
        assert!(f
            .accounts
            .status()
            .iter()
            .any(|s| s.account_id == a.id && s.retired && s.runtime.reserved == 1));
        let unaffected = f
            .resolver
            .resolve_playback(&other, &AudioSelection::Auto, 0)
            .await
            .unwrap();
        drop(unaffected);
        f.released(&b.id).await;

        let rebound = f.connect(&a.id).await;
        assert_eq!(rebound.local_bound_revision, Some(edited.config_revision));
        f.counts(&rebound, 0, 0).await;
        assert!(f.resolver.validate_handle(&old).await.is_err());
        assert!(f
            .resolver
            .resolve_playback(&old, &AudioSelection::Auto, 0)
            .await
            .is_err());
        f.counts(&rebound, 0, 0).await; // no dispatch to the replacement instance
        let fresh_page = f.page(&rebound, LegacySourceList::Tracks, None, 0).await;
        let fresh = SourceHandle::parse(&fresh_page.items[0].id).unwrap();
        assert_ne!(fresh.to_handle(), old.to_handle());
        let fresh_descriptor = f
            .resolver
            .resolve_playback(&fresh, &AudioSelection::Auto, 0)
            .await
            .unwrap();
        drop(fresh_descriptor);
        f.counts(&rebound, 1, 1).await;
        assert_eq!(f.reserved(&a.id), 1); // only the old descriptor still owns a lease
        drop(old_descriptor);
        f.released(&a.id).await;
    })
    .await;
}

#[tokio::test]
async fn unsupported_v1_operations_reject_before_guest_calls_and_do_not_fabricate_metadata() {
    bounded(async {
        let f = Fixture::new().await;
        let row = f.create("A").await;
        let row = f.connect(&row.id).await;
        let page = f.page(&row, LegacySourceList::Tracks, None, 0).await;
        let source = SourceHandle::parse(&page.items[0].id).unwrap();
        f.counts(&row, 0, 1).await;
        for (selection, offset) in [
            (AudioSelection::Auto, 1),
            (
                AudioSelection::Explicit {
                    media_source_id: "real-version".into(),
                    audio_stream_index: 0,
                },
                0,
            ),
        ] {
            let error = f
                .resolver
                .resolve_playback(&source, &selection, offset)
                .await
                .err()
                .unwrap();
            assert_eq!(
                error.to_string(),
                "source v1 does not support offsets or audio selection"
            );
        }
        assert_eq!(
            f.resolver
                .track_handle(&source)
                .await
                .err()
                .unwrap()
                .to_string(),
            "source v1 requires browse metadata when adding tracks"
        );
        assert_eq!(
            f.resolver
                .audio_options_handle(&source)
                .await
                .err()
                .unwrap()
                .to_string(),
            "source v1 does not support audio selection"
        );
        for (offset, limit) in [(u64::from(u32::MAX) + 1, 1), (0, 0)] {
            let error = f
                .resolver
                .extension_legacy_media(
                    &row.id,
                    LegacySourceList::Tracks,
                    None,
                    Some(QUERY.into()),
                    offset,
                    limit,
                )
                .await
                .err()
                .unwrap();
            assert_eq!(error.to_string(), "invalid source v1 browse range");
        }
        f.counts(&row, 0, 1).await; // neither a scan nor a stream was attempted
        let descriptor = f
            .resolver
            .resolve_playback(&source, &AudioSelection::Auto, 0)
            .await
            .unwrap();
        f.counts(&row, 1, 1).await; // positive control: the same real export is usable
        drop(descriptor);
        f.released(&row.id).await;
    })
    .await;
}
