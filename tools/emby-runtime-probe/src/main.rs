use music_player_transport::{HttpReader, HttpRequest};
use rockbox_playback::{Metadata, OutputConfig, Player, PlayerConfig, StreamSession};
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
        json!({"ms": start.elapsed().as_millis(), "event": event, "detail":detail})
    );
    io::stdout().flush().unwrap();
}
fn snapshot(
    start: Instant,
    session: &StreamSession,
    transport: &music_player_transport::TransportHandle,
) -> bool {
    let s = session.snapshot();
    let t = transport.snapshot();
    let output = session.output_snapshot();
    emit(
        start,
        "session",
        json!({"generation":s.generation, "phase":format!("{:?}",s.phase),
        "read_end":format!("{:?}",s.read_end), "cancel_requested":s.cancel_requested,
        "reader_released":s.reader_released,"decoder_joined":s.decoder_joined,
        "decoder_status":s.decoder_status,"transport_terminal":format!("{:?}",t.terminal),
        "worker_exited":t.worker_exited,
        "output_generation": output.generation, "output_boundary":format!("{:?}",output.boundary),
        "output_frames":output.frames, "output_sample_rate":output.sample_rate,
        "output_duration_ns":output.duration().map(|d| d.as_nanos())}),
    );
    s.reader_released && s.decoder_joined && t.worker_exited
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 3);
    let start = Instant::now();
    let output: OutputConfig = args[2].parse().unwrap();
    let player = Player::with_config(PlayerConfig {
        output,
        buffer_seconds: 1.0,
        sample_rate: Some(44100),
        ..Default::default()
    })
    .unwrap();
    let mut request = HttpRequest::new(args[1].clone());
    if args[1].contains("header_positive") || args[1].ends_with("/redirect.mp3") {
        request
            .headers
            .insert("X-Fixture-Auth", "synthetic-sentinel".parse().unwrap());
    }
    let (reader, transport) = HttpReader::start(request).unwrap();
    let cancel = transport.clone();
    let session = player.play_stream(
        Box::new(reader),
        "mp3".into(),
        Metadata::default(),
        move || cancel.cancel(),
    );
    emit(
        start,
        "play_called",
        json!({"boundary":"byte-stream-delivery", "generation":session.generation()}),
    );
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
    let mut released = None;
    loop {
        if let Ok(command) = rx.try_recv() {
            match command.as_str() {
                "stop" => {
                    emit(start, "stop_called", json!({}));
                    player.stop();
                    stopping = Some(Instant::now());
                }
                "pause" => {
                    emit(start, "pause_called", json!({}));
                    player.pause();
                    stopping = Some(Instant::now());
                }
                "drop" => {
                    emit(start, "stop_called", json!({"command":"drop"}));
                    break;
                }
                _ => {}
            }
        }
        let out = player.output_snapshot();
        emit(
            start,
            "output",
            json!({
                "nonblocking":out.nonblocking, "generation":out.generation, "epoch":out.epoch,
                "backpressure_events":out.backpressure_events, "bytes_written":out.bytes_written,
                "nonzero_bytes_written":out.nonzero_bytes_written,
                "output_failed":out.output_failed, "writer_exited":out.writer_exited,
            }),
        );
        let status = player.status();
        emit(
            start,
            "status",
            json!({"state":format!("{:?}",status.state),"index":status.index,
            "position_ms":status.position.as_millis(),"duration_ms":status.duration.as_millis()}),
        );
        if snapshot(start, &session, &transport) && stopping.is_some() && released.is_none() {
            emit(start, "resources_released", json!({}));
            released = Some(Instant::now());
        }
        if released.is_some_and(|at: Instant| at.elapsed() >= Duration::from_millis(250)) {
            break;
        }
        thread::sleep(Duration::from_millis(25));
    }
    emit(start, "drop_started", json!({}));
    drop(player);
    emit(start, "drop_joined", json!({}));
    snapshot(start, &session, &transport);
}
