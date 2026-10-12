//! Deterministic byte-writer accounting; these do not certify receiver consumption.
use super::*;
use crate::managed::StreamInput;
use std::io;
use std::sync::mpsc;

struct PartialWriter {
    bytes: Vec<u8>,
    accept: usize,
    writable: Arc<AtomicBool>,
    blocked: Option<mpsc::Sender<()>>,
}

impl Write for PartialWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.accept > 0 {
            let count = self.accept.min(bytes.len());
            self.bytes.extend_from_slice(&bytes[..count]);
            self.accept -= count;
            return Ok(count);
        }
        if !self.writable.load(Ordering::SeqCst) {
            if let Some(blocked) = self.blocked.take() {
                blocked.send(()).unwrap();
            }
            return Err(io::ErrorKind::WouldBlock.into());
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn register(shared: &Shared, generation: u64) -> (u64, StreamSession) {
    let input = StreamInput::new(
        Box::new(io::empty()),
        "mp3".into(),
        Metadata::default(),
        generation,
        || {},
        || {},
    );
    let session = input.session.clone();
    session.configure_output(44100, StreamOutputBoundary::ByteStream);
    let mut ring = shared.ring.lock().unwrap();
    ring.clear();
    shared
        .output_generation
        .store(generation, Ordering::Relaxed);
    shared.target_amp.store(1f32.to_bits(), Ordering::Relaxed);
    *shared.output_session.lock().unwrap() = Some(session.clone());
    (
        shared.output_epoch.fetch_add(1, Ordering::Relaxed) + 1,
        session,
    )
}

#[test]
fn cancelled_partial_frame_cannot_leak_into_next_generation() {
    // Exercise cancellation at each byte boundary inside an S16LE stereo frame.
    for accepted in 1..8 {
        let shared = make_shared(&PlayerConfig::default(), 44100);
        let (old_epoch, old_session) = register(&shared, 1);
        shared.ring.lock().unwrap().extend([101, 102]);
        let writable = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let (blocked_tx, blocked_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let mut writer = PartialWriter {
            bytes: Vec::new(),
            accept: accepted,
            writable: Arc::clone(&writable),
            blocked: Some(blocked_tx),
        };
        let worker_shared = Arc::clone(&shared);
        let worker_stop = Arc::clone(&stop);
        let worker = std::thread::spawn(move || {
            let mut wire_offset = 0;
            let result = write_output_chunk(
                &mut writer,
                &[1, 2, 3, 4, 5, 6, 7, 8],
                2,
                old_epoch,
                &worker_shared,
                &worker_stop,
                &mut wire_offset,
            );
            finished_tx.send((result, writer, wire_offset)).unwrap();
        });
        blocked_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        // Whole frames count even when a later frame is blocked; a prefix does not.
        assert_eq!(old_session.output_snapshot().frames, (accepted / 4) as u64);
        // The old writer has accepted a frame prefix and is now backpressured.
        shared.revoke_output(1);
        assert!(shared.ring.lock().unwrap().is_empty());
        assert_eq!(shared.target_amp.load(Ordering::Relaxed), 0f32.to_bits());
        let (new_epoch, new_session) = register(&shared, 2);
        assert_eq!(new_session.output_snapshot().frames, 0);
        shared.ring.lock().unwrap().extend([201, 202]);
        // A late cancellation of generation 1 must leave generation 2 untouched.
        shared.revoke_output(1);
        assert_eq!(shared.output_epoch.load(Ordering::Relaxed), new_epoch);
        assert_eq!(shared.output_generation.load(Ordering::Relaxed), 2);
        assert_eq!(shared.target_amp.load(Ordering::Relaxed), 1f32.to_bits());
        assert_eq!(shared.ring.lock().unwrap().len(), 2);
        // Make the destination writable before waiting for old pending output.
        writable.store(true, Ordering::SeqCst);
        let finished = finished_rx.recv_timeout(Duration::from_secs(2));
        if finished.is_err() {
            stop.store(true, Ordering::Relaxed);
        }
        worker.join().unwrap();
        let (result, mut writer, mut wire_offset) = finished.unwrap();
        assert!(!result.unwrap(), "old pending output must be discarded");
        assert_eq!(writer.bytes, [1, 2, 3, 4, 5, 6, 7][..accepted]);
        assert_eq!(wire_offset, accepted % 4);

        assert!(write_output_chunk(
            &mut writer,
            &[21, 22, 23, 24],
            1,
            new_epoch,
            &shared,
            &stop,
            &mut wire_offset,
        )
        .unwrap());
        let mut expected = vec![0; accepted.div_ceil(4) * 4];
        expected[..accepted].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7][..accepted]);
        expected.extend([21, 22, 23, 24]);
        assert_eq!(writer.bytes, expected);
        assert_eq!(wire_offset, 0);
        assert_eq!(
            shared.output_bytes.load(Ordering::Relaxed),
            expected.len() as u64
        );
        assert_eq!(old_session.output_snapshot().frames, (accepted / 4) as u64);
        assert_eq!(new_session.output_snapshot().frames, 1);
        assert_eq!(old_session.output_snapshot().generation, 1);
        assert_eq!(new_session.output_snapshot().generation, 2);
        assert_eq!(
            shared.output_nonzero_bytes.load(Ordering::Relaxed),
            accepted as u64 + 4
        );
        assert!(shared.output_backpressure.load(Ordering::Relaxed) > 0);
    }
}

#[test]
fn output_stop_discards_backpressured_pending_bytes() {
    let shared = make_shared(&PlayerConfig::default(), 44100);
    let (epoch, session) = register(&shared, 1);
    let stop = Arc::new(AtomicBool::new(false));
    let writable = Arc::new(AtomicBool::new(false));
    let (blocked_tx, blocked_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let worker_stop = Arc::clone(&stop);
    let worker = std::thread::spawn(move || {
        let mut writer = PartialWriter {
            bytes: Vec::new(),
            accept: 1,
            writable,
            blocked: Some(blocked_tx),
        };
        let mut offset = 0;
        let result = write_output_chunk(
            &mut writer,
            &[1, 2, 3, 4],
            1,
            epoch,
            &shared,
            &worker_stop,
            &mut offset,
        );
        finished_tx.send((result, writer.bytes, offset)).unwrap();
    });
    blocked_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    stop.store(true, Ordering::Relaxed);
    let (result, bytes, offset) = finished_rx.recv_timeout(Duration::from_secs(2)).unwrap();
    worker.join().unwrap();
    assert!(!result.unwrap());
    assert_eq!(bytes, [1]);
    assert_eq!(offset, 1);
    assert_eq!(session.output_snapshot().frames, 0);
}

#[test]
fn accepted_silence_counts_only_when_it_is_media() {
    struct ByteWriter {
        frames_before_write: Vec<u64>,
        session: StreamSession,
    }
    impl Write for ByteWriter {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.frames_before_write
                .push(self.session.output_snapshot().frames);
            Ok(bytes.len().min(1))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let shared = make_shared(&PlayerConfig::default(), 44100);
    let (epoch, session) = register(&shared, 1);
    let mut writer = ByteWriter {
        frames_before_write: Vec::new(),
        session: session.clone(),
    };
    let stop = AtomicBool::new(false);
    // Two alignment bytes, one silent media frame, then one underrun frame.
    // All bytes are zero; signal amplitude must never decide the media clock.
    let mut wire_offset = 2;
    assert!(write_output_chunk(
        &mut writer,
        &[0; 8],
        1,
        epoch,
        &shared,
        &stop,
        &mut wire_offset
    )
    .unwrap());
    assert_eq!(writer.frames_before_write, [0, 0, 0, 0, 0, 0, 1, 1, 1, 1]);
    assert_eq!(session.output_snapshot().frames, 1);
    assert_eq!(shared.output_bytes.load(Ordering::Relaxed), 10);
    assert!(write_output_chunk(
        &mut writer,
        &[0; 8],
        0,
        epoch,
        &shared,
        &stop,
        &mut wire_offset
    )
    .unwrap());
    assert_eq!(session.output_snapshot().frames, 1);
    assert_eq!(wire_offset, 0);
}

#[test]
fn output_duration_preserves_long_sessions_and_unknown_boundary() {
    let mut clock = StreamOutputSnapshot {
        generation: 7,
        boundary: StreamOutputBoundary::ByteStream,
        frames: 44100 * 7_200 + 22_050,
        sample_rate: 44100,
    };
    assert_eq!(clock.duration(), Some(Duration::from_millis(7_200_500)));
    clock.frames = u64::MAX;
    clock.sample_rate = 1;
    assert_eq!(clock.duration(), Some(Duration::from_secs(u64::MAX)));
    clock.sample_rate = 0;
    assert_eq!(clock.duration(), None);
    clock.sample_rate = 44100;
    clock.boundary = StreamOutputBoundary::Unavailable;
    assert_eq!(clock.duration(), None);
}
