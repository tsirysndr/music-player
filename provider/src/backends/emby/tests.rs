//! Loopback protocol tests only. All credentials and media metadata are synthetic.
//! No decoder, media server, or detached fixture task is involved.

mod browse;

use super::{Emby, MusicProvider, Page, ProviderConfig, ProviderError, ResourceKind, SourceRef};
use crate::emby_playback::AudioSelection;
use serde_json::{json, Value};
use std::{
    cell::Cell,
    collections::BTreeMap,
    future::{poll_fn, Future},
    sync::Arc,
    task::Poll,
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
use url::Url;

const TOKEN: &str = "synthetic-emby-token";
const DEVICE: &str = "fixture-device-1";
const ACCOUNT: &str = "saved-fixture-account";
const SERVER: &str = "fixture-server";
const USER: &str = "fixture-user";
const ITEM_PATH: &str = "/Users/fixture-user/Items/55508";
const LIST_PATH: &str = "/Users/fixture-user/Items";
const INFO_PATH: &str = "/Items/55508/PlaybackInfo";
const DELETE_PATH: &str = "/Videos/ActiveEncodings";
const DEADLINE: Duration = Duration::from_secs(10);

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(DEADLINE, future)
        .await
        .expect("loopback operation exceeded its deadline")
}

struct Request {
    method: String,
    target: String,
    headers: BTreeMap<String, String>,
    body: Vec<u8>,
}

impl Request {
    fn url(&self) -> Url {
        assert!(self.target.starts_with('/'));
        Url::parse(&format!("http://fixture.invalid{}", self.target)).unwrap()
    }

    fn query(&self) -> BTreeMap<String, String> {
        unique_query(&self.url())
    }

    fn authenticated(&self) {
        // Do not print full headers, even for synthetic fixtures.
        assert!(self
            .headers
            .get("x-emby-token")
            .is_some_and(|value| value == TOKEN));
        assert!(self
            .headers
            .get("x-emby-authorization")
            .is_some_and(|value| { value.contains(&format!("DeviceId=\"{DEVICE}\"")) }));
    }
}

fn unique_query(url: &Url) -> BTreeMap<String, String> {
    let mut query = BTreeMap::new();
    for (key, value) in url.query_pairs() {
        assert!(
            query.insert(key.into_owned(), value.into_owned()).is_none(),
            "duplicate query key"
        );
    }
    query
}

// This parser is deliberately limited to the client's HTTP/1.1 requests with a
// Content-Length. The entire read, including the body, has one finite deadline.
async fn read_request(stream: &mut TcpStream) -> Request {
    bounded(async {
        let mut bytes = Vec::new();
        let mut buffer = [0; 2048];
        let header_end = loop {
            if let Some(end) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                break end + 4;
            }
            assert!(bytes.len() < 64 * 1024, "oversized fixture request headers");
            let count = stream.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "request ended before its headers");
            bytes.extend_from_slice(&buffer[..count]);
        };
        let head = std::str::from_utf8(&bytes[..header_end]).unwrap();
        let mut lines = head.split("\r\n");
        let mut words = lines.next().unwrap().split_ascii_whitespace();
        let method = words.next().unwrap().to_owned();
        let target = words.next().unwrap().to_owned();
        assert_eq!(words.next(), Some("HTTP/1.1"));
        assert!(words.next().is_none());
        let mut headers = BTreeMap::new();
        for line in lines.filter(|line| !line.is_empty()) {
            let (name, value) = line.split_once(':').expect("invalid request header");
            assert!(headers
                .insert(name.to_ascii_lowercase(), value.trim().to_owned())
                .is_none());
        }
        assert!(!headers.contains_key("transfer-encoding"));
        let length = headers
            .get("content-length")
            .map(|value| value.parse::<usize>().unwrap())
            .unwrap_or(0);
        assert!(length <= 64 * 1024, "oversized fixture request body");
        while bytes.len() < header_end + length {
            let count = stream.read(&mut buffer).await.unwrap();
            assert_ne!(count, 0, "request body was truncated");
            bytes.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(bytes.len(), header_end + length);
        Request {
            method,
            target,
            headers,
            body: bytes[header_end..].to_vec(),
        }
    })
    .await
}

struct Fixture {
    listener: TcpListener,
    requests: Cell<usize>,
}

impl Fixture {
    async fn new() -> Self {
        Self {
            listener: bounded(TcpListener::bind(("127.0.0.1", 0))).await.unwrap(),
            requests: Cell::new(0),
        }
    }

    fn base(&self) -> String {
        format!("http://{}", self.listener.local_addr().unwrap())
    }

    async fn reply(
        &self,
        method: &str,
        path: &str,
        status: u16,
        location: Option<&str>,
        body: &[u8],
    ) -> Request {
        bounded(async {
            let (mut stream, _) = self.listener.accept().await.unwrap();
            let request = read_request(&mut stream).await;
            self.requests.set(self.requests.get() + 1);
            assert_eq!(request.method, method);
            assert_eq!(request.url().path(), path);
            let mut head = format!("HTTP/1.1 {status} Fixture\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n", body.len());
            if let Some(location) = location {
                assert!(!location.contains(['\r', '\n']));
                head.push_str(&format!("Location: {location}\r\n"));
            }
            head.push_str("\r\n");
            stream.write_all(head.as_bytes()).await.unwrap();
            stream.write_all(body).await.unwrap();
            stream.shutdown().await.unwrap();
            request
        }).await
    }

    async fn json(&self, method: &str, path: &str, value: Value) -> Request {
        self.reply(
            method,
            path,
            200,
            None,
            &serde_json::to_vec(&value).unwrap(),
        )
        .await
    }

    async fn idle(&self, expected_requests: usize) {
        assert_eq!(self.requests.get(), expected_requests);
        // One immediate poll checks the accept backlog after the client future
        // has completed; it does not sleep for an arbitrary quiet interval.
        let pending_request = bounded(poll_fn(|cx| {
            Poll::Ready(match self.listener.poll_accept(cx) {
                Poll::Ready(Ok(_)) => true,
                Poll::Ready(Err(_)) => panic!("loopback listener failed"),
                Poll::Pending => false,
            })
        }))
        .await;
        assert!(
            !pending_request,
            "client made an unexpected additional request"
        );
    }
}

async fn authenticate(fixture: &Fixture, fallback: bool, follow: bool) -> Arc<Emby> {
    let config = ProviderConfig {
        id: ACCOUNT.into(),
        kind: "emby".into(),
        name: "Fixture".into(),
        url: fixture.base(),
        username: Some("family".into()),
        password: Some(String::new()),
    };
    let (result, ()) = bounded(async {
        tokio::join!(Emby::authenticate(&config, DEVICE, follow), async {
            let mut response = json!({
                "AccessToken": TOKEN, "User": {"Id": USER}, "ServerId": SERVER,
            });
            if fallback {
                response.as_object_mut().unwrap().remove("ServerId");
            }
            let request = fixture
                .json("POST", "/Users/AuthenticateByName", response)
                .await;
            let credentials: Value = serde_json::from_slice(&request.body).unwrap();
            assert_eq!(credentials, json!({"Username":"family", "Pw":""}));
            assert!(!request.headers.contains_key("x-emby-token"));
            assert!(request
                .headers
                .get("x-emby-authorization")
                .is_some_and(|value| { value.contains(&format!("DeviceId=\"{DEVICE}\"")) }));
            if fallback {
                fixture
                    .json("GET", "/System/Info", json!({"Id": SERVER}))
                    .await
                    .authenticated();
            }
        })
    })
    .await;
    let client = result.unwrap();
    assert_eq!(client.identity().server_id, SERVER);
    assert_eq!(client.identity().user_id, USER);
    Arc::new(client)
}

fn episode() -> Value {
    json!({
        "Id":"55508", "Name":"My First Ever Ender Dragon Fight! [First Ever Minecraft Playthrough Ep.40]",
        "Type":"Episode", "MediaType":"Video", "IsFolder":false,
        "RunTimeTicks":191429666670_u64, "ParentIndexNumber":2025050473_u32,
        "IndexNumber":41, "SeriesName":"Synthetic series",
    })
}

fn playback_info(session: &str, default_index: i32) -> Value {
    json!({
        "PlaySessionId": session,
        "MediaSources": [{
            "Id":"mediasource_55508", "RunTimeTicks":191429666670_u64,
            "DefaultAudioStreamIndex":default_index, "ETag":"fixture-version-1",
            "MediaStreams":[
                {"Index":0, "Type":"Video", "Codec":"h264"},
                {"Index":1, "Type":"Audio", "Codec":"aac", "Channels":2, "SampleRate":44100, "IsDefault":false},
                {"Index":2, "Type":"Audio", "Codec":"aac", "Channels":2, "SampleRate":44100, "IsDefault":true},
            ],
        }],
    })
}

async fn resolve_requests(fixture: &Fixture, info: Value) {
    fixture
        .json("GET", ITEM_PATH, episode())
        .await
        .authenticated();
    let request = fixture.json("GET", INFO_PATH, info).await;
    request.authenticated();
    assert_eq!(
        request.query(),
        BTreeMap::from([
            ("DeviceId".into(), DEVICE.into()),
            ("UserId".into(), USER.into()),
        ])
    );
}

async fn cleanup_request(fixture: &Fixture, session: &str, status: u16) {
    let request = fixture
        .reply("DELETE", DELETE_PATH, status, None, b"")
        .await;
    request.authenticated();
    assert_eq!(
        request.query(),
        BTreeMap::from([
            ("DeviceId".into(), DEVICE.into()),
            ("PlaySessionId".into(), session.into()),
        ])
    );
}

#[tokio::test]
async fn empty_password_authentication_uses_authenticated_server_identity_fallback() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, true, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    assert_eq!(source.account_id, ACCOUNT);
    assert_eq!(source.remote, *client.identity());
    assert_eq!(SourceRef::parse(&source.to_handle()).unwrap(), source);
    assert!(!source.to_handle().contains(TOKEN));
    fixture.idle(2).await;
}

#[tokio::test]
async fn listing_exceeds_500_and_advances_by_actual_server_capped_page_size() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    const TOTAL: usize = 617;
    const CAP: usize = 137;
    let (result, ()) = bounded(async {
        tokio::join!(client.tracks(None, Page::all()), async {
            for start in (0..TOTAL).step_by(CAP) {
                let items: Vec<_> = (start..(start + CAP).min(TOTAL))
                    .map(|index| {
                        json!({
                            "Id":format!("item-{index}"), "Name":format!("Leaf {index}"),
                            "MediaType":if index % 2 == 0 { "Audio" } else { "Video" },
                            "IsFolder":false,
                        })
                    })
                    .collect();
                let request = fixture
                    .json(
                        "GET",
                        LIST_PATH,
                        json!({
                            "Items":items, "StartIndex":start, "TotalRecordCount":TOTAL,
                        }),
                    )
                    .await;
                request.authenticated();
                let query = request.query();
                assert_eq!(query.get("StartIndex"), Some(&start.to_string()));
                assert_eq!(query.get("Limit").map(String::as_str), Some("500"));
                assert_eq!(
                    query.get("MediaTypes").map(String::as_str),
                    Some("Audio,Video")
                );
                assert_eq!(query.get("Recursive").map(String::as_str), Some("true"));
            }
        })
    })
    .await;
    let tracks = result.unwrap();
    assert_eq!(tracks.len(), TOTAL);
    for (index, track) in tracks.iter().enumerate() {
        let source = SourceRef::parse(&track.id).unwrap();
        assert_eq!(source.item_id, format!("item-{index}"));
        assert_eq!(source.account_id, ACCOUNT);
        assert_eq!(source.remote, *client.identity());
        assert_eq!(track.id, track.uri);
        assert!(track.album.is_none());
    }
    fixture.idle(6).await;
}

#[tokio::test]
async fn episode_movie_and_audio_without_album_remain_playable_leaf_metadata() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    for item in [
        episode(),
        json!({"Id":"movie","Name":"Film","Type":"Movie","MediaType":"Video","IsFolder":false}),
        json!({"Id":"audio","Name":"Recording","Type":"Audio","MediaType":"Audio","IsFolder":false}),
    ] {
        let id = item["Id"].as_str().unwrap();
        let source = client.reference(ResourceKind::Item, id).unwrap();
        let handle = source.to_handle();
        let path = format!("{LIST_PATH}/{id}");
        let (result, request) = bounded(async {
            tokio::join!(
                client.track(&handle),
                fixture.json("GET", &path, item.clone())
            )
        })
        .await;
        request.authenticated();
        let track = result.unwrap();
        assert_eq!(track.id, handle);
        assert_eq!(track.uri, handle);
        assert!(track.album.is_none());
        if id == "55508" {
            assert_eq!(track.title, item["Name"].as_str().unwrap());
            assert_eq!(track.disc_number, 0);
            assert_eq!(track.track_number, None);
            assert!(track.duration.unwrap() > 19_000.0);
        }
    }
    fixture.idle(4).await;
}

#[tokio::test]
async fn cross_origin_302_keeps_token_and_signed_query_and_can_be_disabled() {
    for follow in [true, false] {
        let origin = Fixture::new().await;
        let target = Fixture::new().await;
        assert_ne!(origin.base(), target.base());
        let client = authenticate(&origin, false, follow).await;
        let handle = client
            .reference(ResourceKind::Item, "55508")
            .unwrap()
            .to_handle();
        let signed_target = "/signed/item?sig=a%2Fb%2Bc&space=one%20two&dup=1&dup=2";
        let location = format!("{}{signed_target}", target.base());
        let (result, ()) = bounded(async {
            tokio::join!(client.track(&handle), async {
                let first = origin
                    .reply("GET", ITEM_PATH, 302, Some(&location), b"")
                    .await;
                first.authenticated();
                assert!(first.query().contains_key("Fields"));
                if follow {
                    let redirected = target.json("GET", "/signed/item", episode()).await;
                    redirected.authenticated();
                    assert_eq!(redirected.target, signed_target);
                    assert!(
                        redirected.headers.get("x-emby-authorization")
                            == first.headers.get("x-emby-authorization")
                    );
                }
            })
        })
        .await;
        if follow {
            assert_eq!(result.unwrap().id, handle);
        } else {
            assert!(matches!(result, Err(ProviderError::Other(message))
                if message == "Emby HTTP redirects are disabled"));
        }
        origin.idle(2).await;
        target.idle(usize::from(follow)).await;
    }
}

#[tokio::test]
async fn resolve_forces_mp3_exact_audio_index_and_checked_ticks_then_releases_only_its_session() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let selection = AudioSelection::Auto;
    let (result, ()) = bounded(async {
        tokio::join!(
            client.resolve_audio(&source, &selection, 7_200_123),
            resolve_requests(&fixture, playback_info("session-exact-offset", 1)),
        )
    })
    .await;
    let resolved = result.unwrap();
    assert_eq!(resolved.url.path(), "/Audio/55508/stream.mp3");
    assert_eq!(
        unique_query(&resolved.url),
        BTreeMap::from([
            ("UserId".into(), USER.into()),
            ("MediaSourceId".into(), "mediasource_55508".into()),
            ("AudioStreamIndex".into(), "1".into()),
            ("AudioCodec".into(), "mp3".into()),
            ("EnableAutoStreamCopy".into(), "false".into()),
            ("StartTimeTicks".into(), "72001230000".into()),
            ("DeviceId".into(), DEVICE.into()),
            ("PlaySessionId".into(), "session-exact-offset".into()),
        ])
    );
    assert_eq!(resolved.url.origin().ascii_serialization(), fixture.base());
    assert!(!resolved.url.as_str().contains(TOKEN));
    assert!(resolved
        .headers
        .get("x-emby-token")
        .is_some_and(|value| value == TOKEN));
    assert!(resolved.follow_redirects);
    assert_eq!(resolved.requested_offset_ms, 7_200_123);
    assert_eq!(resolved.pin.audio_stream_index, 1);
    assert_eq!(resolved.pin.runtime_ticks, Some(191429666670));
    // This checks the transient request descriptor, not decoding or the
    // server's actual seek start. No stream GET should occur during resolution.
    let (released, ()) = bounded(async {
        tokio::join!(
            resolved.lease.release(),
            cleanup_request(&fixture, "session-exact-offset", 204)
        )
    })
    .await;
    released.unwrap();
    assert_eq!(Arc::strong_count(&client), 1);
    fixture.idle(4).await;
}

// Each implicit-cleanup test owns the only other Arc. Waiting for that task's
// client reference to disappear observes completion after the DELETE response,
// rather than ending the runtime immediately after the fixture writes it.
async fn cleanup_finished(client: &Arc<Emby>) {
    bounded(async {
        while Arc::strong_count(client) > 1 {
            tokio::task::yield_now().await;
        }
    })
    .await;
}

#[tokio::test]
async fn failed_selection_after_session_allocation_releases_that_session() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let selection = AudioSelection::Explicit {
        media_source_id: "mediasource_55508".into(),
        audio_stream_index: 99,
    };
    let (result, ()) = bounded(async {
        tokio::join!(client.resolve_audio(&source, &selection, 0), async {
            resolve_requests(&fixture, playback_info("session-selection-failed", 1)).await;
            cleanup_request(&fixture, "session-selection-failed", 204).await;
        })
    })
    .await;
    assert!(matches!(result, Err(ProviderError::Other(message))
        if message == "the selected Emby audio stream is unavailable"));
    cleanup_finished(&client).await;
    fixture.idle(4).await;
}

#[tokio::test]
async fn lease_dropped_on_non_tokio_thread_still_releases_its_session() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let selection = AudioSelection::Auto;
    let (result, ()) = bounded(async {
        tokio::join!(
            client.resolve_audio(&source, &selection, 0),
            resolve_requests(&fixture, playback_info("session-thread-drop", 1))
        )
    })
    .await;
    let lease = result.unwrap().lease;
    let worker = std::thread::spawn(move || {
        assert!(tokio::runtime::Handle::try_current().is_err());
        drop(lease);
    });
    // Never block this runtime on an unfinished native thread. The worker only
    // drops its lease; no fixture or socket wait lives on that thread.
    bounded(async {
        while !worker.is_finished() {
            tokio::task::yield_now().await;
        }
    })
    .await;
    worker.join().expect("lease-drop thread panicked");
    cleanup_request(&fixture, "session-thread-drop", 204).await;
    cleanup_finished(&client).await;
    fixture.idle(4).await;
}

#[tokio::test]
async fn resume_keeps_original_pin_when_server_default_changes() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let mut selection = AudioSelection::Auto;
    for (session, default_index, offset) in [
        ("session-pin-first", 1, 0),
        ("session-pin-resume", 2, 12_345),
    ] {
        let (result, ()) = bounded(async {
            tokio::join!(
                client.resolve_audio(&source, &selection, offset),
                resolve_requests(&fixture, playback_info(session, default_index))
            )
        })
        .await;
        let resolved = result.unwrap();
        assert_eq!(resolved.pin.audio_stream_index, 1);
        assert_eq!(resolved.requested_offset_ms, offset);
        assert_eq!(
            unique_query(&resolved.url)
                .get("AudioStreamIndex")
                .map(String::as_str),
            Some("1")
        );
        if let AudioSelection::Pinned(previous) = &selection {
            assert_eq!(&resolved.pin, previous);
        }
        selection = AudioSelection::Pinned(resolved.pin.clone());
        let (released, ()) = bounded(async {
            tokio::join!(
                resolved.lease.release(),
                cleanup_request(&fixture, session, 204)
            )
        })
        .await;
        released.unwrap();
    }
    assert_eq!(Arc::strong_count(&client), 1);
    fixture.idle(7).await;
}

#[tokio::test]
async fn foreign_or_unknown_identity_and_overflowing_offsets_fail_before_network() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let mut foreign_account = source.clone();
    foreign_account.account_id = "another-saved-account".into();
    let mut foreign_server = source.clone();
    foreign_server.remote.server_id = "another-server".into();
    let mut foreign_user = source.clone();
    foreign_user.remote.user_id = "another-user".into();
    let mut unknown_identity = source.clone();
    unknown_identity.remote.server_id.clear();
    let mut container = source.clone();
    container.kind = ResourceKind::Container;
    let selection = AudioSelection::Auto;
    // No request handler runs here. Unexpected network access fails the bounded
    // call or leaves a queued connection detected below; it cannot satisfy Err.
    for invalid in [
        foreign_account,
        foreign_server,
        foreign_user,
        unknown_identity,
        container,
    ] {
        assert!(bounded(client.resolve_audio(&invalid, &selection, 0))
            .await
            .is_err());
        fixture.idle(1).await;
    }
    for offset in [u64::MAX, i64::MAX as u64 / 10_000 + 1] {
        let result = bounded(client.resolve_audio(&source, &selection, offset)).await;
        assert!(matches!(result, Err(ProviderError::Other(message))
            if message == "Emby seek offset is out of range"));
        fixture.idle(1).await;
    }
}

#[tokio::test]
async fn cleanup_retries_the_owned_session_after_transient_http_failure() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let source = client.reference(ResourceKind::Item, "55508").unwrap();
    let selection = AudioSelection::Auto;
    let (result, ()) = bounded(async {
        tokio::join!(
            client.resolve_audio(&source, &selection, 0),
            resolve_requests(&fixture, playback_info("session-retry", 1))
        )
    })
    .await;
    let lease = result.unwrap().lease;
    let (released, ()) = bounded(async {
        tokio::join!(lease.release(), async {
            cleanup_request(&fixture, "session-retry", 500).await;
            cleanup_request(&fixture, "session-retry", 503).await;
            cleanup_request(&fixture, "session-retry", 204).await;
        })
    })
    .await;
    released.unwrap();
    assert_eq!(Arc::strong_count(&client), 1);
    fixture.idle(6).await;
}

#[tokio::test]
async fn repeated_pages_fail_instead_of_duplicating_or_looping_forever() {
    let fixture = Fixture::new().await;
    let client = authenticate(&fixture, false, true).await;
    let (result, ()) = bounded(async {
        tokio::join!(client.tracks(None, Page::all()), async {
            for start in [0, 1] {
                fixture
                    .json(
                        "GET",
                        LIST_PATH,
                        json!({
                            "Items":[episode()], "StartIndex":start, "TotalRecordCount":3,
                        }),
                    )
                    .await
                    .authenticated();
            }
        })
    })
    .await;
    assert!(matches!(result, Err(ProviderError::Other(message))
        if message == "Emby listing repeated an item; retry the listing"));
    fixture.idle(3).await;
}
