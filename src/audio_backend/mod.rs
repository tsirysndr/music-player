use thiserror::Error;

use crate::{config::AudioFormat, convert::Converter, decoder::AudioPacket};

use self::rodio::RodioSink;

#[derive(Debug, Error)]
pub enum SinkError {
    #[error("Audio Sink Error Not Connected: {0}")]
    NotConnected(String),
    #[error("Audio Sink Error Connection Refused: {0}")]
    ConnectionRefused(String),
    #[error("Audio Sink Error On Write: {0}")]
    OnWrite(String),
    #[error("Audio Sink Error Invalid Parameters: {0}")]
    InvalidParams(String),
    #[error("Audio Sink Error Changing State: {0}")]
    StateChange(String),
}

pub type SinkResult<T> = Result<T, SinkError>;

pub trait Open {
    fn open(_: Option<String>, format: AudioFormat) -> Self;
}

pub trait Sink {
    fn start(&mut self) -> SinkResult<()> {
        Ok(())
    }
    fn stop(&mut self) -> SinkResult<()> {
        Ok(())
    }
    fn write(
        &mut self,
        packet: AudioPacket,
        channels: u16,
        sample_rate: u32,
        converter: &mut Converter,
    ) -> SinkResult<()>;
}

pub type SinkBuilder = fn(Option<String>, AudioFormat) -> Box<dyn Sink>;

fn mk_sink<S: Sink + Open + 'static>(device: Option<String>, format: AudioFormat) -> Box<dyn Sink> {
    Box::new(S::open(device, format))
}

pub mod rodio;

pub mod sdl;

pub const BACKENDS: &[(&str, SinkBuilder)] = &[
    (RodioSink::NAME, rodio::mk_rodio), // default goes first
];

pub fn find(name: Option<String>) -> Option<SinkBuilder> {
    if let Some(name) = name {
        BACKENDS
            .iter()
            .find(|backend| name == backend.0)
            .map(|backend| backend.1)
    } else {
        BACKENDS.first().map(|backend| backend.1)
    }
}
