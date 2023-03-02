#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Arc;

use async_graphql::{Request, Response, Schema};
use futures::{
    future::Either::{Left, Right},
    stream::StreamExt,
};
use music_player_addons::{CurrentDevice, CurrentReceiverDevice, CurrentSourceDevice};
use music_player_graphql::{
    scan_devices,
    schema::{
        objects::{player_state::PlayerState, track::Track},
        playback::PositionMilliseconds,
        Mutation, Query, Subscription,
    },
    simple_broker::SimpleBroker,
    MusicPlayerSchema,
};
use music_player_playback::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    player::{Player, PlayerEvent},
};
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use tauri::Manager;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

mod graphql_server;

use graphql_server::run_graphql_server;

#[derive(Debug, Clone, PartialEq, Eq)]
enum ClientSubscriptionEvent {
    Unsubscribed,
}

#[tauri::command]
fn execute_graphql_subscription(
    request: Request,
    schema: tauri::State<'_, MusicPlayerSchema>,
    app_handle: tauri::AppHandle,
) -> Result<Uuid, ()> {
    let token = Uuid::new_v4();
    let (client_subscription_tx, client_subscription_rx) = futures::channel::mpsc::unbounded();
    let graphql_stream = schema.execute_stream(request);
    let cloned_handle = app_handle.clone();
    tokio::spawn(async move {
        let mut combined_stream =
            futures::stream::select(graphql_stream.map(Left), client_subscription_rx.map(Right));
        loop {
            match combined_stream.next().await {
                None => {
                    break;
                }
                Some(Right(ClientSubscriptionEvent::Unsubscribed)) => {
                    break;
                }
                Some(Left(data)) => {
                    cloned_handle
                        .emit_all(
                            &format!("subscriptions/{}", token),
                            // Needs to be serialized first since Response doesn't implement Clone
                            serde_json::to_value(&data).unwrap(),
                        )
                        .ok();
                }
            }
        }
    });
    app_handle.once_global(format!("unsubscribe/{}", token), move |_| {
        client_subscription_tx
            .unbounded_send(ClientSubscriptionEvent::Unsubscribed)
            .ok();
    });
    Ok(token)
}

#[tauri::command]
async fn execute_graphql(
    request: Request,
    schema: tauri::State<'_, MusicPlayerSchema>,
) -> Result<Response, ()> {
    Ok(schema.execute(request).await)
}

#[tokio::main]
async fn main() {
    let audio_format = AudioFormat::default();
    let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();
    let tracklist = Arc::new(std::sync::Mutex::new(Tracklist::new_empty()));
    let devices = scan_devices().await.unwrap();
    let current_device = Arc::new(Mutex::new(CurrentDevice::new()));
    let source_device = Arc::new(Mutex::new(CurrentSourceDevice::new()));
    let receiver_device = Arc::new(Mutex::new(CurrentReceiverDevice::new()));
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let cmd_tx = Arc::new(std::sync::Mutex::new(cmd_tx));
    let cmd_rx = Arc::new(std::sync::Mutex::new(cmd_rx));

    let (_, _) = Player::new(
        move || backend(None, audio_format),
        move |event| match event {
            PlayerEvent::CurrentTrack {
                track,
                position,
                position_ms,
                is_playing,
            } => {
                if let Some(track) = track.clone() {
                    SimpleBroker::publish(Track::from(track));
                    SimpleBroker::publish(PlayerState {
                        index: position as u32,
                        position_ms,
                        is_playing,
                    });
                }
            }
            PlayerEvent::TrackTimePosition { position_ms } => {
                SimpleBroker::publish(PositionMilliseconds { position_ms });
            }
            _ => {}
        },
        Arc::clone(&cmd_tx),
        Arc::clone(&cmd_rx),
        tracklist.clone(),
    );
    let db = Arc::new(Mutex::new(Database::new().await));
    let schema: MusicPlayerSchema = Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .data(db)
    .data(cmd_tx)
    .data(tracklist)
    .data(devices)
    .data(current_device)
    .data(source_device)
    .data(receiver_device)
    .finish();

    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    if settings.tauri_enable_graphql_server {
        tokio::spawn(run_graphql_server(schema.clone()));
    }

    tauri::async_runtime::set(tokio::runtime::Handle::current());
    tauri::Builder::default()
        .manage(schema)
        .invoke_handler(tauri::generate_handler![
            execute_graphql,
            execute_graphql_subscription
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
