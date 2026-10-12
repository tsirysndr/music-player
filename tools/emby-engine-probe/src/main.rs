//! Public-API baseline only. No emulated engine, application migration or real credentials.
use rockbox_playback::{OutputConfig, Player, PlayerConfig};
use serde_json::json;
use std::{
    io::{self, BufRead, Write},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

fn emit(start: Instant, event: &str, detail: serde_json::Value) {
    println!(
        "{}",
        json!({"ms": start.elapsed().as_millis(), "event": event, "detail": detail})
    );
    io::stdout().flush().unwrap();
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(
        args.len(),
        3,
        "usage: emby-engine-baseline URL tcp-connect:127.0.0.1:PORT"
    );
    let start = Instant::now();
    let output: OutputConfig = args[2].parse().unwrap();
    let player = Player::with_config(PlayerConfig {
        output,
        buffer_seconds: 1.0,
        sample_rate: Some(44100),
        ..Default::default()
    })
    .unwrap();
    emit(
        start,
        "constructed",
        json!({"version": "rockbox-playback 0.7.0", "boundary": "tcp-delivery"}),
    );
    player.set_queue([args[1].clone()]);
    player.play();
    emit(start, "play_called", json!({}));
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in io::stdin().lock().lines() {
            let Ok(line) = line else { break };
            if tx.send(line).is_err() {
                break;
            }
        }
    });
    let mut stopping = None;
    loop {
        if let Ok(command) = rx.try_recv() {
            match command.as_str() {
                "stop" => {
                    player.stop();
                    emit(start, "stop_called", json!({}));
                    stopping = Some(Instant::now());
                }
                "pause" => {
                    player.pause();
                    emit(start, "pause_called", json!({}));
                }
                "play" => {
                    player.play();
                    emit(start, "play_called", json!({}));
                }
                "finish" => break,
                _ => {}
            }
        }
        let status = player.status();
        emit(
            start,
            "status",
            json!({"state": format!("{:?}", status.state), "index": status.index, "position_ms": status.position.as_millis(), "duration_ms": status.duration.as_millis()}),
        );
        if stopping.is_some_and(|at: Instant| at.elapsed() >= Duration::from_millis(2250)) {
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    emit(start, "drop_started", json!({}));
    drop(player);
    emit(start, "drop_joined", json!({}));
}
