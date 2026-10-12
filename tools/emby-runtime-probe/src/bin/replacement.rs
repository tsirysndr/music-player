//! Public Player replacement child. The controller bounds every command by 5s.
//! stdin: {id:u64,command:play_a|replace_b|cancel_a|snapshot|drop_b}, one per line.
//! Every command emits command_called, state, command_returned with the same id.
//! drop_b additionally emits drop_started/drop_joined. No detached probe threads.
use music_player_transport::{HttpReader, HttpRequest, TransportHandle};
use rockbox_playback::{Metadata, OutputConfig, Player, PlayerConfig, StreamSession};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use std::time::{Duration, Instant};

struct Events {
    start: Instant,
    seq: u64,
}
impl Events {
    fn emit(&mut self, event: &str, detail: Value) -> io::Result<()> {
        self.seq += 1;
        let mut out = io::stdout().lock();
        writeln!(
            out,
            "{}",
            json!({
                "seq": self.seq, "ms": self.start.elapsed().as_millis(),
                "event": event, "detail": detail
            })
        )?;
        out.flush()
    }
}

struct Session {
    stream: StreamSession,
    transport: TransportHandle,
}
impl Session {
    fn start(player: &Player, url: &str) -> io::Result<Self> {
        let mut request = HttpRequest::new(url.to_owned());
        request.connect_timeout = Duration::from_secs(3);
        // The stalled-header case must finish by cancellation, not this timeout.
        request.header_timeout = Duration::from_secs(60);
        request.read_timeout = Duration::from_secs(60);
        let (reader, transport) = HttpReader::start(request)?;
        let wake = transport.clone();
        let stream = player.play_stream(
            Box::new(reader),
            "mp3".into(),
            Metadata::default(),
            move || wake.cancel(),
        );
        Ok(Self { stream, transport })
    }

    fn snapshot(&self) -> Value {
        let s = self.stream.snapshot();
        let t = self.transport.snapshot();
        let o = self.stream.output_snapshot();
        json!({
            "generation": s.generation, "phase": format!("{:?}", s.phase),
            "read_end": format!("{:?}", s.read_end),
            "cancel_requested": s.cancel_requested, "reader_released": s.reader_released,
            "decoder_joined": s.decoder_joined, "decoder_status": s.decoder_status,
            "transport_terminal": format!("{:?}", t.terminal),
            "worker_exited": t.worker_exited,
            "output_generation": o.generation, "output_boundary":format!("{:?}",o.boundary),
            "output_frames":o.frames, "output_sample_rate":o.sample_rate,
            "output_duration_ns":o.duration().map(|d| d.as_nanos())
        })
    }
}

fn state(player: Option<&Player>, a: Option<&Session>, b: Option<&Session>, id: u64) -> Value {
    let output = player.map(|p| {
        let o = p.output_snapshot();
        json!({
            "generation": o.generation, "epoch": o.epoch,
            "nonblocking": o.nonblocking, "bytes_written": o.bytes_written,
            "nonzero_bytes_written": o.nonzero_bytes_written,
            "backpressure_events": o.backpressure_events,
            "output_failed": o.output_failed, "writer_exited": o.writer_exited
        })
    });
    json!({"id": id, "A": a.map(Session::snapshot), "B": b.map(Session::snapshot),
        "output": output})
}

fn run(events: &mut Events) -> io::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 4 {
        return Err(io::Error::other("expected A URL, B URL, TCP output"));
    }
    let output: OutputConfig = args[3]
        .parse()
        .map_err(|_| io::Error::other("invalid output configuration"))?;
    let mut player = Some(
        Player::with_config(PlayerConfig {
            output,
            sample_rate: Some(44100),
            buffer_seconds: 1.0,
            ..Default::default()
        })
        .map_err(|_| io::Error::other("Player construction failed"))?,
    );
    events.emit(
        "ready",
        json!({
            "protocol": 1, "sample_rate": player.as_ref().unwrap().sample_rate(),
            "channels": 2, "format": "s16le", "boundary": "TCP byte-stream delivery"
        }),
    )?;
    let (mut a, mut b): (Option<Session>, Option<Session>) = (None, None);
    let mut last_id = 0;
    let mut stale_cancelled = false;
    for line in io::stdin().lock().lines() {
        let line = line?;
        if line.len() > 4096 {
            return Err(io::Error::other("oversized command"));
        }
        let command: Value =
            serde_json::from_str(&line).map_err(|_| io::Error::other("invalid command JSON"))?;
        let id = command["id"]
            .as_u64()
            .filter(|id| *id > last_id)
            .ok_or_else(|| io::Error::other("command ids must increase"))?;
        let name = command["command"]
            .as_str()
            .ok_or_else(|| io::Error::other("missing command"))?;
        last_id = id;
        events.emit("command_called", json!({"id":id, "command":name}))?;
        match name {
            "play_a" if a.is_none() && b.is_none() => {
                a = Some(Session::start(player.as_ref().unwrap(), &args[1])?);
            }
            "replace_b" if a.is_some() && b.is_none() => {
                // This public call itself must replace/cancel A; no pre-cancel here.
                b = Some(Session::start(player.as_ref().unwrap(), &args[2])?);
            }
            "cancel_a" if b.is_some() && !stale_cancelled => {
                a.as_ref().unwrap().stream.cancel();
                stale_cancelled = true;
            }
            "snapshot" => {}
            "drop_b" if b.is_some() && stale_cancelled => {
                b.as_ref().unwrap().stream.cancel();
                events.emit("drop_started", json!({"id":id}))?;
                drop(player.take());
                events.emit("drop_joined", json!({"id":id}))?;
            }
            _ => return Err(io::Error::other("invalid command order")),
        }
        events.emit("state", state(player.as_ref(), a.as_ref(), b.as_ref(), id))?;
        events.emit("command_returned", json!({"id":id, "command":name}))?;
        if name == "drop_b" {
            return Ok(());
        }
    }
    Err(io::Error::other("controller closed before drop_b"))
}

fn main() {
    let mut events = Events {
        start: Instant::now(),
        seq: 0,
    };
    if let Err(error) = run(&mut events) {
        let _ = events.emit("fatal", json!({"reason":error.to_string()}));
        std::process::exit(1);
    }
}
