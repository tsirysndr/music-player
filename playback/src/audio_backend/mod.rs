use pipe::StdoutSink;
use subprocess::SubprocessSink;
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

pub trait SinkAsBytes {
    fn write_bytes(&mut self, data: &[u8]) -> SinkResult<()>;
}

pub mod pipe;
pub mod rodio;
pub mod sdl;
pub mod subprocess;

pub const BACKENDS: &[(&str, SinkBuilder)] = &[
    (RodioSink::NAME, rodio::mk_rodio), // default goes first
    (StdoutSink::NAME, mk_sink::<StdoutSink>),
    (SubprocessSink::NAME, mk_sink::<SubprocessSink>),
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

#[macro_export]
macro_rules! sink_as_bytes {
    () => {
        fn write(
            &mut self,
            packet: AudioPacket,
            _channels: u16,
            _sample_rate: u32,
            converter: &mut Converter,
        ) -> SinkResult<()> {
            use crate::convert::i24;
            use zerocopy::AsBytes;
            match packet {
                AudioPacket::Samples(samples) => match self.format {
                    AudioFormat::F64 => self.write_bytes(samples.as_bytes()),
                    AudioFormat::F32 => {
                        let samples_f32: &[f32] = &converter.f64_to_f32(&samples);
                        self.write_bytes(samples_f32.as_bytes())
                    }
                    AudioFormat::S32 => {
                        let samples_s32: &[i32] = &converter.f64_to_s32(&samples);
                        self.write_bytes(samples_s32.as_bytes())
                    }
                    AudioFormat::S24 => {
                        let samples_s24: &[i32] = &converter.f64_to_s24(&samples);
                        self.write_bytes(samples_s24.as_bytes())
                    }
                    AudioFormat::S24_3 => {
                        let samples_s24_3: &[i24] = &converter.f64_to_s24_3(&samples);
                        self.write_bytes(samples_s24_3.as_bytes())
                    }
                    AudioFormat::S16 => {
                        let samples_s16: &[i16] = &converter.f64_to_s16(&samples);
                        self.write_bytes(samples_s16.as_bytes())
                    }
                },
                AudioPacket::Raw(samples) => self.write_bytes(&samples),
            }
        }
    };
}
