use std::fs::File;

use clap::Command;
use music_player::{
    audio_backend::{self, rodio::RodioSink},
    config::AudioFormat,
    convert::Converter,
    decoder::{symphonia_decoder::SymphoniaDecoder, AudioDecoder},
    dither::{mk_ditherer, TriangularDitherer},
};
use std::path::Path;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
mod formatter;
mod output;

use log::error;

type Decoder = Box<dyn AudioDecoder + Send>;

fn cli() -> Command<'static> {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    Command::new("music-player")
        .version(VERSION)
        .author("Tsiry Sandratraina <tsiry.sndr@aol.com>")
        .about("A simple music player written in Rust")
        .subcommand(
            Command::new("play")
                .about("Play a song")
                .arg_from_usage("<song> 'The path to the song'"),
        )
}

#[tokio::main]
async fn main() {
    let matches = cli().get_matches();

    if let Some(matches) = matches.subcommand_matches("play") {
        let song = matches.value_of("song").unwrap();

        formatter::print_format(song);

        let audio_format = AudioFormat::default();
        let backend = audio_backend::find(Some(RodioSink::NAME.to_string())).unwrap();

        // Create a hint to help the format registry guess what format reader is appropriate.
        let mut hint = Hint::new();

        let path = Path::new(song);

        // Provide the file extension as a hint.
        if let Some(extension) = path.extension() {
            if let Some(extension_str) = extension.to_str() {
                hint.with_extension(extension_str);
            }
        }

        let source = Box::new(File::open(path).unwrap());

        // Create the media source stream using the boxed media source from above.
        let mss = MediaSourceStream::new(source, Default::default());

        let symphonia_decoder = |mss: MediaSourceStream, hint| {
            SymphoniaDecoder::new(mss, hint).map(|mut decoder| {
                // For formats other that Vorbis, we'll try getting normalisation data from
                // ReplayGain metadata fields, if present.
                Box::new(decoder) as Decoder
            })
        };

        let decoder_type = symphonia_decoder(mss, hint);

        let mut decoder = match decoder_type {
            Ok(decoder) => decoder,
            Err(e) => {
                error!("Failed to create decoder: {}", e);
                return;
            }
        };

        let mut sink = backend(None, audio_format);

        sink.start();

        loop {
            match decoder.next_packet() {
                Ok(result) => {
                    if let Some((ref _packet_position, packet, channels, sample_rate)) = result {
                        match packet.samples() {
                            Ok(_) => {
                                // println!("packet_position: {:?}", packet_position);
                                // println!("packet: {:?}", packet.samples());
                                let mut converter =
                                    Converter::new(Some(mk_ditherer::<TriangularDitherer>));
                                sink.write(packet, channels, sample_rate, &mut converter);
                            }
                            Err(e) => {
                                error!("Failed to decode packet: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to decode packet: {}", e);
                }
            };
        }
    }
}
