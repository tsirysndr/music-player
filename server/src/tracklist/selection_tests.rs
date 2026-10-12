//! Real protobuf/tonic round trips, with the actual queue but no playback engine.
use super::*;
use crate::api::music::v1alpha1::{
    tracklist_service_client::TracklistServiceClient,
    tracklist_service_server::TracklistServiceServer, SelectedMediaEntry,
};
use migration::{Migrator, MigratorTrait};
use music_player_storage::saved_servers::{self, NewServer};
use music_player_types::{
    audio::AudioPin,
    source::{RemoteIdentity, ResourceKind},
};
use std::{future::Future, sync::Mutex, time::Duration};
use tokio::{
    net::TcpListener,
    sync::mpsc::{self, UnboundedReceiver},
};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    transport::{Channel, Server},
    Code,
};

async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(15), future)
        .await
        .expect("selection RPC deadline")
}

#[tokio::test]
async fn full_metadata_play_rpc_preserves_choice_and_mode_in_one_command() {
    bounded(async {
        let mut f = Fixture::new().await;
        for next in [false, true] {
            let entry = f.entry(Some(MediaQueueChoice {
                media_source_id: Some("version-b".into()),
                audio_stream_index: Some(0),
            }));
            let expected = entry.track.clone().unwrap();
            let (response, id) = tokio::join!(
                f.client.play_selected_media(PlaySelectedMediaRequest {
                    entry: Some(entry),
                    next
                }),
                async {
                    let PlayerCommand::PlaySelectedMedia {
                        track,
                        selection,
                        next: actual,
                        reply,
                    } = f.commands.recv().await.unwrap()
                    else {
                        panic!("expected a single play command");
                    };
                    assert_eq!(actual, next);
                    assert_eq!(track.id, expected.id);
                    assert_eq!(track.title, expected.title);
                    assert_eq!(
                        selection,
                        AudioSelection::Explicit {
                            media_source_id: "version-b".into(),
                            audio_stream_index: 0
                        }
                    );
                    // This checks RPC dispatch and real occurrence allocation;
                    // the player command test separately verifies activation.
                    let id = f
                        .queue
                        .lock()
                        .unwrap()
                        .insert_next_with_selection(track, selection)
                        .unwrap();
                    reply.send(Ok(id.clone())).unwrap();
                    id
                }
            );
            assert_eq!(response.unwrap().into_inner().occurrence_id, id);
            f.no_commands();
        }
        let malformed = f.entry(Some(MediaQueueChoice {
            media_source_id: Some("version-b".into()),
            audio_stream_index: None,
        }));
        let error = f
            .client
            .play_selected_media(PlaySelectedMediaRequest {
                entry: Some(malformed),
                next: false,
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
        let error = f
            .client
            .play_selected_media(PlaySelectedMediaRequest {
                entry: None,
                next: false,
            })
            .await
            .unwrap_err();
        assert_eq!(error.code(), Code::InvalidArgument);
        f.no_commands();
    })
    .await;
}

struct Fixture {
    client: TracklistServiceClient<Channel>,
    commands: UnboundedReceiver<PlayerCommand>,
    queue: Arc<Mutex<TracklistState>>,
    source: SourceRef,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for Fixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

impl Fixture {
    async fn new() -> Self {
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let row = saved_servers::upsert(
            db.get_connection(),
            &NewServer::new("emby", "Synthetic", "http://127.0.0.1:1")
                .with_credentials(Some("family".into()), Some(String::new())),
            "2026-10-01T00:00:00Z",
        )
        .await
        .unwrap();
        let source = SourceRef {
            resolver: "emby".into(),
            account_id: row.id.clone(),
            remote: RemoteIdentity {
                server_id: "server".into(),
                user_id: "family".into(),
            },
            kind: ResourceKind::Item,
            item_id: "episode".into(),
        };
        saved_servers::bind_remote_identity(db.get_connection(), &row, &source.remote)
            .await
            .unwrap();
        let queue = Arc::new(Mutex::new(TracklistState::new_empty()));
        let (tx, commands) = mpsc::unbounded_channel();
        let service = Tracklist::new(queue.clone(), Arc::new(Mutex::new(tx)), db);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move {
            Server::builder()
                .add_service(TracklistServiceServer::new(service))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        // Construct the guard before awaiting the connection, so a deadline
        // or panic cannot leave a detached server behind.
        let channel = Channel::from_shared(endpoint).unwrap().connect_lazy();
        Self {
            client: TracklistServiceClient::new(channel),
            commands,
            queue,
            source,
            server,
        }
    }

    fn entry(&self, choice: Option<MediaQueueChoice>) -> SelectedMediaEntry {
        SelectedMediaEntry {
            track: Some(Track {
                id: self.source.to_handle(),
                uri: self.source.to_handle(),
                title: "Original episode".into(),
                ..Default::default()
            }),
            choice,
        }
    }

    fn no_commands(&mut self) {
        assert!(matches!(
            self.commands.try_recv(),
            Err(mpsc::error::TryRecvError::Empty)
        ));
    }

    async fn close(mut self) {
        self.server.abort();
        let _ = (&mut self.server).await;
    }
}

fn explicit(index: i32) -> Option<MediaQueueChoice> {
    Some(MediaQueueChoice {
        media_source_id: Some("original".into()),
        audio_stream_index: Some(index),
    })
}

#[tokio::test]
async fn duplicate_items_keep_distinct_occurrences_and_exact_accepted_ticks_on_wire() {
    bounded(async {
        let mut f = Fixture::new().await;
        let request = AddSelectedMediaRequest {
            entries: vec![f.entry(None), f.entry(explicit(0))],
        };
        let mut client = f.client.clone();
        let queue = f.queue.clone();
        let apply = async {
            let PlayerCommand::LoadSelectedTracks {
                tracks,
                start_index,
                reply,
            } = f.commands.recv().await.unwrap()
            else {
                panic!("expected selected append")
            };
            assert_eq!(start_index, None);
            assert_eq!(tracks[0].1, AudioSelection::Auto);
            assert_eq!(
                tracks[1].1,
                AudioSelection::Explicit {
                    media_source_id: "original".into(),
                    audio_stream_index: 0
                }
            );
            let ids = queue.lock().unwrap().queue_with_selection(tracks).unwrap();
            reply.send(Ok(ids)).unwrap();
        };
        let (response, ()) = tokio::join!(client.add_selected_media(request), apply);
        let ids = response.unwrap().into_inner().occurrence_ids;
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
        {
            let mut queue = f.queue.lock().unwrap();
            queue.next_track();
            queue
                .accept_current_pin(
                    &ids[0],
                    AudioPin {
                        media_source_id: "original".into(),
                        audio_stream_index: 0,
                        runtime_ticks: Some(u64::MAX - 1),
                        etag: Some("not-an-api-field".into()),
                        codec: Some("aac".into()),
                        channels: Some(2),
                        sample_rate: Some(44100),
                    },
                )
                .unwrap();
        }
        let snapshot = f
            .client
            .get_media_queue(GetMediaQueueRequest {})
            .await
            .unwrap()
            .into_inner();
        let current = snapshot.current.unwrap();
        assert_eq!(current.occurrence_id, ids[0]);
        assert_eq!(current.choice.unwrap().audio_stream_index, None);
        let pin = current.accepted_audio.unwrap();
        assert_eq!(pin.runtime_ticks, Some(u64::MAX - 1));
        assert_eq!(pin.audio_stream_index, 0);
        assert_eq!(pin.codec.as_deref(), Some("aac"));
        assert_eq!(pin.channels, Some(2));
        assert_eq!(pin.sample_rate, Some(44100));
        assert_eq!(snapshot.played[0].occurrence_id, ids[0]);
        assert_eq!(snapshot.upcoming[0].occurrence_id, ids[1]);
        assert_eq!(
            snapshot.upcoming[0]
                .choice
                .as_ref()
                .unwrap()
                .audio_stream_index,
            Some(0)
        );
        assert!(snapshot.upcoming[0].accepted_audio.is_none());
        f.no_commands();
        f.close().await;
    })
    .await;
}

#[tokio::test]
async fn bad_choice_or_source_rejects_complete_batch_without_commands() {
    bounded(async {
        let mut f = Fixture::new().await;
        let mut invalid = vec![
            f.entry(Some(MediaQueueChoice {
                media_source_id: Some("original".into()),
                audio_stream_index: None,
            })),
            f.entry(Some(MediaQueueChoice {
                media_source_id: None,
                audio_stream_index: Some(0),
            })),
            f.entry(Some(MediaQueueChoice {
                media_source_id: Some("  ".into()),
                audio_stream_index: Some(0),
            })),
            f.entry(explicit(-1)),
            SelectedMediaEntry {
                track: None,
                choice: None,
            },
        ];
        for handle in ["mp-source:v9?bad".to_owned(), {
            let mut container = f.source.clone();
            container.kind = ResourceKind::Container;
            container.to_handle()
        }] {
            let mut entry = f.entry(None);
            let track = entry.track.as_mut().unwrap();
            track.id = handle.clone();
            track.uri = handle;
            invalid.push(entry);
        }
        for bad in invalid {
            let request = AddSelectedMediaRequest {
                entries: vec![f.entry(None), bad],
            };
            let err = f.client.add_selected_media(request).await.unwrap_err();
            assert_eq!(err.code(), Code::InvalidArgument);
            f.no_commands();
        }
        let mut wrong = f.source.clone();
        wrong.remote.user_id = "other-user".into();
        let mut bad = f.entry(None);
        let track = bad.track.as_mut().unwrap();
        track.id = wrong.to_handle();
        track.uri = wrong.to_handle();
        let request = AddSelectedMediaRequest {
            entries: vec![f.entry(None), bad],
        };
        let err = f.client.add_selected_media(request).await.unwrap_err();
        assert_eq!(err.code(), Code::FailedPrecondition);
        f.no_commands();
        assert!(f
            .client
            .add_selected_media(AddSelectedMediaRequest { entries: vec![] })
            .await
            .unwrap()
            .into_inner()
            .occurrence_ids
            .is_empty());
        f.no_commands();
        assert!(f.queue.lock().unwrap().is_empty());
        f.close().await;
    })
    .await;
}

#[tokio::test]
async fn select_pair_zero_auto_and_player_rejection_are_not_silently_dropped() {
    bounded(async {
        let mut f = Fixture::new().await;
        let err = f
            .client
            .select_media_audio(SelectMediaAudioRequest {
                occurrence_id: "stale-occurrence".into(),
                choice: Some(MediaQueueChoice {
                    media_source_id: None,
                    audio_stream_index: Some(0),
                }),
            })
            .await
            .unwrap_err();
        assert_eq!(err.code(), Code::InvalidArgument);
        f.no_commands();
        for choice in [explicit(0), None] {
            let expected = if choice.is_some() {
                AudioSelection::Explicit {
                    media_source_id: "original".into(),
                    audio_stream_index: 0,
                }
            } else {
                AudioSelection::Auto
            };
            let mut client = f.client.clone();
            let apply = async {
                let PlayerCommand::SelectAudio {
                    occurrence_id,
                    selection,
                    reply,
                } = f.commands.recv().await.unwrap()
                else {
                    panic!("expected selection")
                };
                assert_eq!(occurrence_id, "stale-occurrence");
                assert_eq!(selection, expected);
                reply
                    .send(Err("queue occurrence no longer exists".into()))
                    .unwrap();
            };
            let (result, ()) = tokio::join!(
                client.select_media_audio(SelectMediaAudioRequest {
                    occurrence_id: "stale-occurrence".into(),
                    choice
                }),
                apply
            );
            assert_eq!(result.unwrap_err().code(), Code::FailedPrecondition);
            f.no_commands();
        }
        f.close().await;
    })
    .await;
}

#[tokio::test]
async fn timeout_refresh_can_be_empty_before_late_append_without_resubmitting() {
    bounded(async {
        let mut f = Fixture::new().await;
        let request = AddSelectedMediaRequest {
            entries: vec![f.entry(None)],
        };
        // Keep the receiver alive but do not acknowledge. The command remains
        // executable after the RPC times out: timeout is not rollback.
        let error = f.client.add_selected_media(request).await.unwrap_err();
        assert_eq!(error.code(), Code::DeadlineExceeded);
        assert!(error.message().contains("outcome uncertain"));
        assert!(error
            .message()
            .contains("A refresh showing no change does not prove failure"));
        assert!(error.message().contains("Do not retry this mutation"));
        // Read through the real RPC before consuming the pending command.
        // This snapshot is not a barrier for Player's command queue.
        let before = f
            .client
            .get_media_queue(GetMediaQueueRequest {})
            .await
            .unwrap()
            .into_inner();
        assert!(before.current.is_none());
        assert!(before.played.is_empty());
        assert!(before.upcoming.is_empty());
        // Simulate late command application using the actual Tracklist. This
        // fixture does not run Player or cover its playback lifecycle.
        let PlayerCommand::LoadSelectedTracks { tracks, reply, .. } =
            f.commands.recv().await.unwrap()
        else {
            panic!("expected one append")
        };
        let ids = f
            .queue
            .lock()
            .unwrap()
            .queue_with_selection(tracks)
            .unwrap();
        assert!(reply.send(Ok(ids.clone())).is_err());
        let snapshot = f
            .client
            .get_media_queue(GetMediaQueueRequest {})
            .await
            .unwrap()
            .into_inner();
        assert_eq!(snapshot.upcoming.len(), 1);
        assert_eq!(snapshot.upcoming[0].occurrence_id, ids[0]);
        f.no_commands();
        f.close().await;
    })
    .await;
}
