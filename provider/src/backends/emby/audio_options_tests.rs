//! Synthetic loopback metadata only. No playback session, decoder or detached
//! server task; all socket operations and each entire test have deadlines.

use super::*;
use music_player_types::audio::AudioOptions;
use serde_json::{json, Value};
use std::{cell::Cell, collections::BTreeMap, future::Future, task::Poll};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

const ITEM_PATH: &str = "/Users/options-user/Items/episode";
const TOKEN: &str = "synthetic-options-token";

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(10), future)
        .await
        .expect("audio-options fixture deadline")
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

    async fn reply(&self, path: &str, body: Value) {
        bounded(async {
            let (mut socket, _) = self.listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let mut chunk = [0; 2048];
            let end = loop {
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    break end + 4;
                }
                assert!(bytes.len() < 16 * 1024);
                let n = socket.read(&mut chunk).await.unwrap();
                assert_ne!(n, 0);
                bytes.extend_from_slice(&chunk[..n]);
            };
            let head = std::str::from_utf8(&bytes[..end]).unwrap();
            let mut lines = head.split("\r\n");
            let mut request = lines.next().unwrap().split_ascii_whitespace();
            let auth = path == "/Users/AuthenticateByName";
            assert_eq!(request.next(), Some(if auth { "POST" } else { "GET" }));
            let url = Url::parse(&format!("http://fixture.invalid{}", request.next().unwrap())).unwrap();
            assert_eq!(url.path(), path);
            let headers: BTreeMap<_, _> = lines
                .filter(|line| !line.is_empty())
                .map(|line| {
                    let (key, value) = line.split_once(':').unwrap();
                    (key.to_ascii_lowercase(), value.trim().to_owned())
                })
                .collect();
            assert!(!headers.contains_key("transfer-encoding"));
            let length: usize = headers.get("content-length").map(|v| v.parse().unwrap()).unwrap_or(0);
            assert!(length <= 16 * 1024);
            while bytes.len() < end + length {
                let n = socket.read(&mut chunk).await.unwrap();
                assert_ne!(n, 0);
                bytes.extend_from_slice(&chunk[..n]);
            }
            assert_eq!(bytes.len(), end + length);
            if auth {
                assert_eq!(serde_json::from_slice::<Value>(&bytes[end..]).unwrap(),
                    json!({"Username":"family","Pw":""}));
            } else {
                // Every non-auth request must be the item GET, with no stream,
                // PlaybackInfo, DELETE, user/session or token query parameter.
                assert_eq!(path, ITEM_PATH);
                assert_eq!(length, 0);
                assert_eq!(url.query_pairs()
                    .map(|(key, value)| (key.into_owned(), value.into_owned()))
                    .collect::<Vec<_>>(),
                    vec![("Fields".to_owned(), "MediaSources,MediaStreams".to_owned())]);
                assert!(headers.get("x-emby-token").is_some_and(|v| v == TOKEN));
            }
            self.requests.set(self.requests.get() + 1);
            let body = serde_json::to_vec(&body).unwrap();
            let head = format!("HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
            socket.write_all(head.as_bytes()).await.unwrap();
            socket.write_all(&body).await.unwrap();
            socket.shutdown().await.unwrap();
        }).await;
    }

    async fn authenticate(&self) -> Emby {
        let config = ProviderConfig {
            id: "options-account".into(),
            kind: "emby".into(),
            name: "Fixture".into(),
            url: format!("http://{}", self.listener.local_addr().unwrap()),
            username: Some("family".into()),
            password: Some(String::new()),
        };
        let (result, ()) = tokio::join!(
            Emby::authenticate(&config, "options-device", true),
            self.reply(
                "/Users/AuthenticateByName",
                json!({
                    "AccessToken":TOKEN, "User":{"Id":"options-user"},
                    "ServerId":"options-server"
                })
            ),
        );
        result.unwrap()
    }

    async fn options(&self, client: &Emby, item: Value) -> Result<AudioOptions, ProviderError> {
        let source = client.reference(ResourceKind::Item, "episode").unwrap();
        let before = self.requests.get();
        let (result, ()) = tokio::join!(client.audio_options(&source), self.reply(ITEM_PATH, item));
        self.idle(before + 1).await;
        result
    }

    async fn idle(&self, count: usize) {
        assert_eq!(self.requests.get(), count);
        // One poll after completion, without a sleep or an unbounded accept.
        let extra = std::future::poll_fn(|cx| {
            Poll::Ready(match self.listener.poll_accept(cx) {
                Poll::Pending => false,
                Poll::Ready(Ok(_)) => true,
                Poll::Ready(Err(_)) => panic!("fixture listener failed"),
            })
        })
        .await;
        assert!(!extra, "unexpected request after metadata lookup");
    }
}

fn item(versions: Value) -> Value {
    json!({"Id":"episode","Name":"Original episode title","Type":"Episode",
        "MediaType":"Video","MediaSources":versions})
}

fn version(id: &str, streams: Value) -> Value {
    json!({"Id":id,"MediaStreams":streams})
}

#[tokio::test]
async fn item_only_options_preserve_actual_labels_indices_and_omit_transport_fields() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = fixture.authenticate().await;
        let versions = json!([{
            "Id":"cut-original","Name":"Original cut / 原版",
            "RunTimeTicks":191429666670_u64,"DefaultAudioStreamIndex":0,
            "Path":"/private/movie.mkv","DirectStreamUrl":"https://invalid/secret",
            "TranscodingUrl":"/transcode?api_key=secret","PlaySessionId":"secret-session",
            "MediaStreams":[
                {"Type":"Video","Index":1},
                {"Type":"Audio","Index":0,"Title":"原始音轨","DisplayTitle":"AAC stereo",
                 "Language":"","Codec":"aac","Channels":2,"SampleRate":44100,"IsDefault":false,
                 "Path":"/private/audio"},
                {"Type":"Subtitle","Index":4},
                {"Type":"Audio","Index":7,"Title":"Commentary","Language":"eng","IsDefault":true}
            ]
        }, {"Id":"cut-other","Name":"","MediaStreams":[{"Type":"Audio","Index":3}]}]);
        let options = fixture.options(&client, item(versions)).await.unwrap();
        assert_eq!(
            options.source,
            client
                .reference(ResourceKind::Item, "episode")
                .unwrap()
                .to_handle()
        );
        assert_eq!(options.versions.len(), 2);
        let first = &options.versions[0];
        assert_eq!(first.id, "cut-original");
        assert_eq!(first.name.as_deref(), Some("Original cut / 原版"));
        assert_eq!(first.runtime_ticks, Some(191429666670));
        assert_eq!(first.default_audio_stream_index, Some(0));
        assert!(first.unavailable_reason.is_none());
        assert_eq!(
            first
                .audio_streams
                .iter()
                .map(|s| s.index)
                .collect::<Vec<_>>(),
            vec![0, 7]
        );
        let audio = &first.audio_streams[0];
        assert_eq!(audio.title.as_deref(), Some("原始音轨"));
        assert_eq!(audio.display_title.as_deref(), Some("AAC stereo"));
        assert_eq!(audio.language.as_deref(), Some(""));
        assert_eq!(audio.codec.as_deref(), Some("aac"));
        assert_eq!(audio.channels, Some(2));
        assert_eq!(audio.sample_rate, Some(44100));
        assert!(!audio.is_default);
        assert!(audio.unavailable_reason.is_none());
        assert!(first.audio_streams[1].is_default);
        assert_eq!(first.audio_streams[1].language.as_deref(), Some("eng"));
        assert_eq!(options.versions[1].name.as_deref(), Some(""));
        assert_eq!(options.versions[1].audio_streams[0].language, None);
        let serialized = serde_json::to_string(&options).unwrap();
        for secret in [
            "/private",
            "secret",
            TOKEN,
            "PlaySessionId",
            "DirectStreamUrl",
            "TranscodingUrl",
        ] {
            assert!(!serialized.contains(secret));
        }
        fixture.idle(2).await;
    })
    .await;
}

#[tokio::test]
async fn wrong_account_remote_kind_and_empty_item_are_rejected_before_http() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = fixture.authenticate().await;
        let valid = client.reference(ResourceKind::Item, "episode").unwrap();
        for part in 0..6 {
            let mut source = valid.clone();
            match part {
                0 => source.account_id = "other-account".into(),
                1 => source.remote.server_id = "other-server".into(),
                2 => source.remote.user_id = "other-user".into(),
                3 => source.kind = ResourceKind::Container,
                4 => source.item_id.clear(),
                _ => source.resolver = "jellyfin".into(),
            }
            assert!(client.audio_options(&source).await.is_err());
        }
        fixture.idle(1).await;
    })
    .await;
}

#[tokio::test]
async fn empty_sources_and_no_audio_are_explicit_and_containers_are_rejected() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = fixture.authenticate().await;
        let mut missing = item(json!([]));
        missing.as_object_mut().unwrap().remove("MediaSources");
        for metadata in [missing, item(json!([]))] {
            assert!(fixture
                .options(&client, metadata)
                .await
                .unwrap()
                .versions
                .is_empty());
        }
        let options = fixture
            .options(
                &client,
                item(json!([
                    version("empty", json!([])),
                    version(
                        "video-only",
                        json!([{"Type":"Video","Index":0},{"Type":"Subtitle","Index":1}])
                    )
                ])),
            )
            .await
            .unwrap();
        assert_eq!(options.versions.len(), 2);
        for version in options.versions {
            assert!(version.audio_streams.is_empty());
            assert!(version.unavailable_reason.is_some());
        }
        let mut container = item(json!([]));
        container["Type"] = json!("Season");
        assert!(fixture.options(&client, container).await.is_err());
        let mut different = item(json!([]));
        different["Id"] = json!("other-item");
        assert!(fixture.options(&client, different).await.is_err());
        let mut non_media = item(json!([]));
        non_media["MediaType"] = json!("Photo");
        assert!(fixture.options(&client, non_media).await.is_err());
    })
    .await;
}

#[tokio::test]
async fn unsupported_versions_and_ambiguous_indices_remain_visible_but_disabled() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = fixture.authenticate().await;
        let audio = json!([{"Type":"Audio","Index":0}]);
        let mut versions = Vec::new();
        for flag in ["IsInfiniteStream", "RequiresOpening", "RequiresClosing"] {
            let mut value = version(flag, audio.clone());
            value[flag] = json!(true);
            versions.push(value);
        }
        versions.extend([
            version("duplicate", audio.clone()),
            version("duplicate", audio.clone()),
            version("", audio.clone()),
            version(" \t", audio),
            version(
                "mixed",
                json!([
                    {"Type":"Audio","Index":2},{"Type":"Audio","Index":2},
                    {"Type":"Audio","Index":-1},{"Type":"Video","Index":0},
                    {"Type":"Audio","Index":0}
                ]),
            ),
        ]);
        let options = fixture
            .options(&client, item(json!(versions)))
            .await
            .unwrap();
        assert_eq!(options.versions.len(), 8);
        for version in &options.versions[..7] {
            assert!(version.unavailable_reason.is_some());
            assert!(version
                .audio_streams
                .iter()
                .all(|s| s.unavailable_reason.is_some()));
        }
        assert_eq!(options.versions[5].id, "");
        let mixed = &options.versions[7];
        assert!(mixed.unavailable_reason.is_none());
        assert_eq!(
            mixed
                .audio_streams
                .iter()
                .map(|s| s.index)
                .collect::<Vec<_>>(),
            vec![2, 2, -1, 0]
        );
        assert!(mixed.audio_streams[..3]
            .iter()
            .all(|s| s.unavailable_reason.is_some()));
        assert!(mixed.audio_streams[3].unavailable_reason.is_none());
    })
    .await;
}

#[tokio::test]
async fn missing_or_null_source_id_and_stream_index_never_default_to_valid_choices() {
    bounded(async {
        let fixture = Fixture::new().await;
        let client = fixture.authenticate().await;
        for broken in [
            json!({"MediaStreams":[{"Type":"Audio","Index":0}]}),
            json!({"Id":null,"MediaStreams":[{"Type":"Audio","Index":0}]}),
            version("missing-index", json!([{"Type":"Audio"}])),
            version("null-index", json!([{"Type":"Audio","Index":null}])),
        ] {
            let error = fixture
                .options(&client, item(json!([broken])))
                .await
                .unwrap_err();
            assert_eq!(error.to_string(), "invalid Emby metadata response");
        }
    })
    .await;
}
