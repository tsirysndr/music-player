//! Host-owned streaming sessions. Cancellation is independent of the engine queue.
use crate::Metadata;
use std::io::{self, Read};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamPhase {
    Queued,
    Opening,
    Decoding,
    Cancelled,
    Failed,
    EndUnconfirmed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamReadEnd {
    Reading,
    Eof,
    Failed,
    Cancelled,
}
#[derive(Debug, Clone)]
pub struct StreamSnapshot {
    pub generation: u64,
    pub phase: StreamPhase,
    pub read_end: StreamReadEnd,
    pub cancel_requested: bool,
    /// Reader destructor (including host transport join) has returned.
    pub reader_released: bool,
    /// Decoder destructor (including codec thread join) has returned.
    pub decoder_joined: bool,
    /// Codec's own exit status, independent of transport/completion evidence.
    pub decoder_status: Option<i32>,
}
/// Where output frames were observed. Neither boundary confirms audibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamOutputBoundary {
    Unavailable,
    ByteStream,
    DeviceBuffer,
}

/// Per-generation post-DSP output, excluding underrun silence and wire padding.
/// This is output time, not a source checkpoint: pitch/rate mapping and receiver
/// consumption belong to the host. Retained sessions keep their own final count.
#[derive(Debug, Clone, Copy)]
pub struct StreamOutputSnapshot {
    pub generation: u64,
    pub boundary: StreamOutputBoundary,
    pub frames: u64,
    pub sample_rate: u32,
}
impl StreamOutputSnapshot {
    pub fn duration(&self) -> Option<Duration> {
        if self.boundary == StreamOutputBoundary::Unavailable || self.sample_rate == 0 {
            return None;
        }
        let rate = u64::from(self.sample_rate);
        Some(
            Duration::from_secs(self.frames / rate)
                + Duration::from_nanos((self.frames % rate) * 1_000_000_000 / rate),
        )
    }
}

struct Inner {
    state: Mutex<StreamSnapshot>,
    output: Mutex<StreamOutputSnapshot>,
    wake: Box<dyn Fn() + Send + Sync>,
    // Only explicit cancellation revokes output. Normal reader EOF/drop
    // must allow already decoded PCM to drain.
    revoke_output: Box<dyn Fn() + Send + Sync>,
}
/// A single generation; never re-used for another source or retry.
#[derive(Clone)]
pub struct StreamSession(Arc<Inner>);
impl StreamSession {
    pub fn snapshot(&self) -> StreamSnapshot {
        self.0.state.lock().unwrap().clone()
    }
    pub fn output_snapshot(&self) -> StreamOutputSnapshot {
        *self.0.output.lock().unwrap()
    }
    pub(crate) fn configure_output(&self, sample_rate: u32, boundary: StreamOutputBoundary) {
        let mut output = self.0.output.lock().unwrap();
        output.sample_rate = sample_rate;
        output.boundary = boundary;
    }
    pub(crate) fn record_output(&self, frames: usize) {
        let mut output = self.0.output.lock().unwrap();
        output.frames = output.frames.saturating_add(frames as u64);
    }
    pub fn generation(&self) -> u64 {
        self.0.state.lock().unwrap().generation
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.state.lock().unwrap().cancel_requested
    }
    pub fn cancel(&self) {
        {
            let mut state = self.0.state.lock().unwrap();
            state.cancel_requested = true;
            // Preserve a previously observed failure/end while still cancelling resources.
            if !matches!(
                state.phase,
                StreamPhase::Failed | StreamPhase::EndUnconfirmed
            ) {
                state.phase = StreamPhase::Cancelled;
            }
        }
        (self.0.revoke_output)();
        (self.0.wake)();
    }
    pub(crate) fn set_phase(&self, phase: StreamPhase) {
        let mut state = self.0.state.lock().unwrap();
        if !state.cancel_requested {
            state.phase = phase;
        }
    }
    pub(crate) fn decoder_joined(&self) {
        self.0.state.lock().unwrap().decoder_joined = true;
    }
    pub(crate) fn decoder_ended(&self, code: Option<i32>) {
        let mut state = self.0.state.lock().unwrap();
        state.decoder_status = code;
        if !state.cancel_requested {
            state.phase = if code != Some(0) || state.read_end == StreamReadEnd::Failed {
                StreamPhase::Failed
            } else {
                StreamPhase::EndUnconfirmed
            };
        }
    }
}
pub(crate) struct StreamInput {
    pub reader: ObservedReader,
    pub format_ext: String,
    pub metadata: Metadata,
    pub session: StreamSession,
}
impl StreamInput {
    pub fn new(
        reader: Box<dyn Read + Send>,
        format_ext: String,
        metadata: Metadata,
        generation: u64,
        wake: impl Fn() + Send + Sync + 'static,
        revoke_output: impl Fn() + Send + Sync + 'static,
    ) -> Self {
        let session = StreamSession(Arc::new(Inner {
            state: Mutex::new(StreamSnapshot {
                generation,
                phase: StreamPhase::Queued,
                read_end: StreamReadEnd::Reading,
                cancel_requested: false,
                reader_released: false,
                decoder_joined: false,
                decoder_status: None,
            }),
            output: Mutex::new(StreamOutputSnapshot {
                generation,
                boundary: StreamOutputBoundary::Unavailable,
                frames: 0,
                sample_rate: 0,
            }),
            wake: Box::new(wake),
            revoke_output: Box::new(revoke_output),
        }));
        Self {
            reader: ObservedReader {
                reader: Some(reader),
                session: session.clone(),
            },
            format_ext,
            metadata,
            session,
        }
    }
}
pub(crate) struct ObservedReader {
    reader: Option<Box<dyn Read + Send>>,
    session: StreamSession,
}
impl Read for ObservedReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.session.is_cancelled() {
            self.session.0.state.lock().unwrap().read_end = StreamReadEnd::Cancelled;
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream cancelled",
            ));
        }
        let result = self.reader.as_mut().unwrap().read(buf);
        let mut state = self.session.0.state.lock().unwrap();
        if state.cancel_requested {
            state.read_end = StreamReadEnd::Cancelled;
            return Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "stream cancelled",
            ));
        }
        match &result {
            Ok(0) => state.read_end = StreamReadEnd::Eof,
            Err(_) => state.read_end = StreamReadEnd::Failed,
            _ => {}
        }
        result
    }
}
impl Drop for ObservedReader {
    fn drop(&mut self) {
        (self.session.0.wake)();
        drop(self.reader.take());
        self.session.0.state.lock().unwrap().reader_released = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    enum Event {
        Read,
        Wake,
        Revoke,
        ReaderDropped { released: bool },
    }
    type Events = Arc<Mutex<Vec<Event>>>;

    struct RecordingReader {
        events: Events,
        session: Arc<Mutex<Option<StreamSession>>>,
        fail_read: bool,
    }
    impl Read for RecordingReader {
        fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
            self.events.lock().unwrap().push(Event::Read);
            if self.fail_read {
                Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "fixture read error",
                ))
            } else {
                Ok(0)
            }
        }
    }
    impl Drop for RecordingReader {
        fn drop(&mut self) {
            let released = self
                .session
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .snapshot()
                .reader_released;
            self.events
                .lock()
                .unwrap()
                .push(Event::ReaderDropped { released });
        }
    }

    fn input(fail_read: bool) -> (StreamInput, Events) {
        let events = Arc::new(Mutex::new(Vec::new()));
        let session = Arc::new(Mutex::new(None));
        let wake_events = events.clone();
        let revoke_events = events.clone();
        let input = StreamInput::new(
            Box::new(RecordingReader {
                events: events.clone(),
                session: session.clone(),
                fail_read,
            }),
            "mp3".into(),
            Metadata::default(),
            1,
            move || wake_events.lock().unwrap().push(Event::Wake),
            move || revoke_events.lock().unwrap().push(Event::Revoke),
        );
        *session.lock().unwrap() = Some(input.session.clone());
        (input, events)
    }

    #[test]
    fn normal_reader_drop_wakes_then_releases_without_revoking_output() {
        // Cover both early reader disposal and normal EOF cleanup.
        for read_to_eof in [false, true] {
            let (mut input, events) = input(false);
            let session = input.session.clone();
            let mut expected = Vec::new();
            if read_to_eof {
                assert_eq!(input.reader.read(&mut [0; 1]).unwrap(), 0);
                expected.push(Event::Read);
            }
            assert!(!session.snapshot().reader_released);
            drop(input);
            expected.extend([Event::Wake, Event::ReaderDropped { released: false }]);
            assert_eq!(*events.lock().unwrap(), expected);
            let snapshot = session.snapshot();
            assert!(snapshot.reader_released);
            assert!(!snapshot.cancel_requested);
            assert_eq!(snapshot.phase, StreamPhase::Queued);
            assert_eq!(
                snapshot.read_end,
                if read_to_eof {
                    StreamReadEnd::Eof
                } else {
                    StreamReadEnd::Reading
                }
            );
        }
    }

    #[test]
    fn cancellation_revokes_before_wake_and_prevents_reader_io() {
        let (mut input, events) = input(false);
        let session = input.session.clone();
        session.set_phase(StreamPhase::Decoding);
        session.cancel();
        assert_eq!(*events.lock().unwrap(), [Event::Revoke, Event::Wake]);
        assert!(session.is_cancelled());
        assert_eq!(session.snapshot().phase, StreamPhase::Cancelled);
        assert!(!session.snapshot().reader_released);
        let error = input.reader.read(&mut [0; 1]).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted);
        assert_eq!(session.snapshot().read_end, StreamReadEnd::Cancelled);
        // No Read event: cancellation must not enter the underlying reader.
        assert_eq!(*events.lock().unwrap(), [Event::Revoke, Event::Wake]);
        session.set_phase(StreamPhase::Decoding);
        session.decoder_ended(Some(0));
        assert_eq!(session.snapshot().phase, StreamPhase::Cancelled);
        drop(input);
        assert!(session.snapshot().reader_released);
        assert_eq!(
            *events.lock().unwrap(),
            [
                Event::Revoke,
                Event::Wake,
                Event::Wake,
                Event::ReaderDropped { released: false },
            ]
        );
    }

    #[test]
    fn cancellation_preserves_failed_and_end_unconfirmed_phases() {
        for (code, expected) in [(1, StreamPhase::Failed), (0, StreamPhase::EndUnconfirmed)] {
            let (mut input, _) = input(false);
            let session = input.session.clone();
            assert_eq!(input.reader.read(&mut [0; 1]).unwrap(), 0);
            session.decoder_ended(Some(code));
            assert_eq!(session.snapshot().phase, expected);
            session.cancel();
            assert!(session.is_cancelled());
            assert_eq!(session.snapshot().phase, expected);
            assert_eq!(
                input.reader.read(&mut [0; 1]).unwrap_err().kind(),
                io::ErrorKind::ConnectionAborted
            );
            // Late decoder/progress updates cannot replace a terminal outcome.
            session.set_phase(StreamPhase::Decoding);
            session.decoder_ended(Some(1 - code));
            assert_eq!(session.snapshot().phase, expected);
        }
    }

    #[test]
    fn successful_decoder_status_cannot_override_read_failure() {
        let (mut input, _) = input(true);
        let session = input.session.clone();
        assert_eq!(
            input.reader.read(&mut [0; 1]).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        assert_eq!(session.snapshot().read_end, StreamReadEnd::Failed);
        session.decoder_ended(Some(0));
        session.decoder_joined();
        drop(input);
        let snapshot = session.snapshot();
        assert_eq!(snapshot.decoder_status, Some(0));
        assert_eq!(snapshot.read_end, StreamReadEnd::Failed);
        assert_eq!(snapshot.phase, StreamPhase::Failed);
        assert!(snapshot.reader_released && snapshot.decoder_joined);
        assert!(!snapshot.cancel_requested);
    }
}
