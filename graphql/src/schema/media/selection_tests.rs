use super::*;
use async_graphql::{EmptySubscription, Request, Schema, Variables};
use migration::{Migrator, MigratorTrait};
use music_player_playback::source_resolver::SourceResolver;
use music_player_storage::saved_servers;
use music_player_storage::Database;
use music_player_types::source::{RemoteIdentity, ResourceKind, SourceRef};
use serde_json::json;

#[tokio::test]
async fn saved_account_batch_preserves_choices_and_returns_player_occurrences() {
    tokio::time::timeout(std::time::Duration::from_secs(15), async {
        let mut options = sea_orm::ConnectOptions::new("sqlite::memory:");
        options.max_connections(1).sqlx_logging(false);
        let connection = sea_orm::Database::connect(options).await.unwrap();
        Migrator::up(&connection, None).await.unwrap();
        let db = Database { connection };
        let row = saved_servers::upsert(db.get_connection(),
            &saved_servers::NewServer::new("emby", "Synthetic", "http://127.0.0.1:1")
                .with_credentials(Some("family".into()), Some(String::new())),
            "2026-10-01T00:00:00Z").await.unwrap();
        let source = SourceRef { resolver: "emby".into(), account_id: row.id.clone(),
            remote: RemoteIdentity { server_id: "server".into(), user_id: "family".into() },
            kind: ResourceKind::Item, item_id: "55508".into() };
        saved_servers::bind_remote_identity(db.get_connection(), &row, &source.remote).await.unwrap();
        let track = json!({"id":source.to_handle(),"uri":source.to_handle(),"title":"Same episode","discNumber":0});
        let resolver = SourceResolver::new(db.clone(), "graphql-selection-fixture".into(), true);
        let (schema, mut commands, queue) = api();
        let mutation = "mutation($entries: [SelectedMediaInput!]!) { addSelectedMedia(entries: $entries) }";
        // The malformed account comes second: rejecting it must not enqueue
        // the first valid entry or require a live server authentication.
        let mut wrong = source.clone();
        wrong.remote.user_id = "another-user".into();
        let response = schema.execute(Request::new(mutation).data(resolver.clone()).data(db.clone())
            .variables(Variables::from_json(json!({"entries":[
                {"track":track.clone()},
                {"track":{"id":wrong.to_handle(),"uri":wrong.to_handle(),"title":"Wrong account","discNumber":0}}
            ]})))).await;
        assert!(!response.errors.is_empty());
        assert!(matches!(commands.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
        let request = Request::new(mutation).data(resolver).data(db).variables(Variables::from_json(json!({"entries":[
            {"track":track.clone(),"choice":{"mediaSourceId":"original","audioStreamIndex":0}},
            {"track":track}
        ]})));
        let (response, ids) = tokio::join!(schema.execute(request), async {
            let PlayerCommand::LoadSelectedTracks { tracks, start_index, reply } = commands.recv().await.unwrap() else {
                panic!("expected selected batch");
            };
            assert!(start_index.is_none());
            assert_eq!(tracks.len(), 2);
            assert_eq!(tracks[0].0.id, source.to_handle());
            assert_eq!(tracks[0].1, audio::AudioSelection::Explicit { media_source_id: "original".into(), audio_stream_index: 0 });
            assert_eq!(tracks[1].1, audio::AudioSelection::Auto);
            // Real Tracklist allocation; command execution is simulated.
            let ids = queue.lock().unwrap().queue_with_selection(tracks).unwrap();
            reply.send(Ok(ids.clone())).unwrap();
            ids
        });
        assert!(response.errors.is_empty(), "{:?}", response.errors);
        assert_eq!(response.data.into_json().unwrap()["addSelectedMedia"], json!(ids));
        assert_ne!(ids[0], ids[1]);
        assert!(matches!(commands.try_recv(), Err(tokio::sync::mpsc::error::TryRecvError::Empty)));
    }).await.expect("saved account selection deadline");
}

fn api() -> (
    Schema<MediaQuery, MediaMutation, EmptySubscription>,
    tokio::sync::mpsc::UnboundedReceiver<PlayerCommand>,
    Arc<StdMutex<Tracklist>>,
) {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let queue = Arc::new(StdMutex::new(Tracklist::new_empty()));
    let schema = Schema::build(MediaQuery, MediaMutation, EmptySubscription)
        .data(Arc::new(StdMutex::new(sender)))
        .data(Arc::new(Mutex::new(CurrentReceiverDevice::new())))
        .data(queue.clone())
        .finish();
    (schema, receiver, queue)
}

#[tokio::test]
async fn malformed_choice_rejects_whole_batch_before_command() {
    let (schema, mut commands, _) = api();
    for choice in [
        json!({"mediaSourceId":"version-a"}),
        json!({"audioStreamIndex":0}),
        json!({"mediaSourceId":"version-a","audioStreamIndex":-1}),
    ] {
        let track = json!({"id":"legacy","uri":"file.mp3","title":"Song","discNumber":0});
        let response = schema.execute(Request::new(
            "mutation($entries: [SelectedMediaInput!]!) { addSelectedMedia(entries: $entries) }"
        ).variables(Variables::from_json(json!({"entries":[
            {"track":track.clone()}, {"track":track,"choice":choice}
        ]})))).await;
        assert_eq!(response.errors.len(), 1);
        assert!(response.errors[0]
            .message
            .contains("An audio choice needs both"));
        assert!(matches!(
            commands.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Empty)
        ));
    }
}

#[tokio::test]
async fn selection_passes_real_zero_index_and_returns_player_rejection() {
    let (schema, mut commands, _) = api();
    let request = Request::new(
        "mutation { selectMediaAudio(occurrenceId: \"removed-occurrence\", choice: {mediaSourceId: \"version-a\", audioStreamIndex: 0}) }"
    );
    let (response, ()) = tokio::time::timeout(std::time::Duration::from_secs(10), async {
        tokio::join!(schema.execute(request), async {
            match commands.recv().await.unwrap() {
                PlayerCommand::SelectAudio {
                    occurrence_id,
                    selection,
                    reply,
                } => {
                    assert_eq!(occurrence_id, "removed-occurrence");
                    assert_eq!(
                        selection,
                        audio::AudioSelection::Explicit {
                            media_source_id: "version-a".into(),
                            audio_stream_index: 0
                        }
                    );
                    reply
                        .send(Err("queue occurrence no longer exists".into()))
                        .unwrap();
                }
                _ => panic!("unexpected player command"),
            }
        })
    })
    .await
    .unwrap();
    assert_eq!(response.errors.len(), 1);
    assert_eq!(
        response.errors[0].message,
        "queue occurrence no longer exists"
    );
}

#[tokio::test]
async fn queue_query_distinguishes_duplicate_items() {
    let (schema, _, queue) = api();
    let track = music_player_entity::track::Model {
        id: "same-song".into(),
        uri: "file.mp3".into(),
        ..Default::default()
    };
    {
        let mut state = queue.lock().unwrap();
        state.queue(vec![track.clone(), track]);
        state.next_track();
    }
    let response = schema.execute("{ mediaQueue { current { occurrenceId } played { occurrenceId track { id } } upcoming { occurrenceId track { id } choice { mediaSourceId audioStreamIndex } acceptedAudio { audioStreamIndex } } } }").await;
    assert!(response.errors.is_empty(), "{:?}", response.errors);
    let data = response.data.into_json().unwrap();
    let queue = &data["mediaQueue"];
    assert_eq!(
        queue["current"]["occurrenceId"],
        queue["played"][0]["occurrenceId"]
    );
    assert_ne!(
        queue["played"][0]["occurrenceId"],
        queue["upcoming"][0]["occurrenceId"]
    );
    assert_eq!(
        queue["played"][0]["track"]["id"],
        queue["upcoming"][0]["track"]["id"]
    );
    assert!(queue["upcoming"][0]["choice"]["audioStreamIndex"].is_null());
    assert!(queue["upcoming"][0]["acceptedAudio"].is_null());
}
