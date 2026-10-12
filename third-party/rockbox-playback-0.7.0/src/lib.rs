//! A small audio playback engine built on Rockbox's own building blocks:
//! [`rockbox-codecs`](https://crates.io/crates/rockbox-codecs) for
//! decoding, [`rockbox-dsp`](https://crates.io/crates/rockbox-dsp) for the
//! full DSP chain (ReplayGain, resampling, 10-band EQ, tone controls,
//! surround, channel mixing, compressor, dither and pitch), and
//! [`cpal`](https://crates.io/crates/cpal) for output.
//!
//! It provides a queue, transport controls, native **ReplayGain**, the
//! complete Rockbox **DSP** feature set (see [`DspSettings`] and the
//! `Player::set_*` setters), and a faithful port of Rockbox's **crossfade**
//! (see [`crossfade`]).
//!
//! ```no_run
//! use rockbox_playback::{Player, CrossfadeSettings};
//!
//! let player = Player::new()?;
//! player.set_crossfade(CrossfadeSettings::always());   // 2 s crossfade
//! player.set_replaygain(rockbox_playback::ReplayGainMode::Track, 0.0, true);
//! player.set_eq_preset(rockbox_playback::EqPreset::Rock); // ready-made EQ curve
//! player.set_queue(vec!["a.flac", "b.mp3", "c.opus"]);
//! player.play();
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Design
//!
//! One background *engine* thread owns the single decode session (Rockbox
//! codecs share one global `codec_api`) and the single DSP instance. It
//! decodes ahead into a ring buffer that the cpal callback drains, so
//! transport controls stay responsive and pause/resume is click-free
//! (the callback applies Rockbox's ~⅓ s volume fade). Because only one
//! decoder can be open at a time, crossfade is done by holding the
//! outgoing track's tail, decoding the incoming track's head, and mixing
//! the two — never by opening two decoders at once.

#[cfg(feature = "http")]
pub mod adaptive;
mod crossfade;
pub mod m3u;
mod managed;
pub use managed::{
    StreamOutputBoundary, StreamOutputSnapshot, StreamPhase, StreamReadEnd, StreamSession,
    StreamSnapshot,
};
pub mod output;
mod resume;
pub mod source;

#[cfg(feature = "http")]
pub use adaptive::AdaptiveStream;
pub use crossfade::{CrossfadeMode, CrossfadeSettings, MixMode};
pub use m3u::M3uEntry;
pub use output::{OutputConfig, ParseOutputError, SocketMode};
pub use resume::ResumeState;
pub use rockbox_codecs::Decoder;
pub use rockbox_metadata::Metadata;
pub use source::{is_url, FileSource, MediaSource};
#[cfg(feature = "http")]
pub use source::{HttpSource, HttpStream, IcyInfo};

/// Read a resume snapshot written by a previous session without constructing
/// a [`Player`] — e.g. to decide whether to offer "resume playback" in a UI.
/// See [`Player::resume`] to actually restore it.
pub fn load_resume(path: impl AsRef<std::path::Path>) -> Option<ResumeState> {
    resume::load(path.as_ref())
}

use std::collections::VecDeque;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{
    AtomicBool, AtomicI32, AtomicU32, AtomicU64, AtomicU8, AtomicUsize, Ordering,
};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[cfg(feature = "cpal")]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

/// How the queue repeats when a track (or the whole queue) finishes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RepeatMode {
    /// No repeat — playback stops after the last track.
    #[default]
    Off,
    /// Repeat the current track indefinitely (on automatic advance; a manual
    /// `next` / `previous` still moves to another track).
    One,
    /// Repeat the whole queue — wrap back to the start after the last track
    /// (and to the end when stepping back from the first).
    All,
}

impl RepeatMode {
    fn to_u8(self) -> u8 {
        match self {
            RepeatMode::Off => 0,
            RepeatMode::One => 1,
            RepeatMode::All => 2,
        }
    }
    fn from_u8(v: u8) -> Self {
        match v {
            1 => RepeatMode::One,
            2 => RepeatMode::All,
            _ => RepeatMode::Off,
        }
    }
}

/// ReplayGain mode — which tag to apply (mirrors `rockbox-dsp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ReplayGainMode {
    /// Off — no gain adjustment.
    #[default]
    Off,
    /// Per-track gain.
    Track,
    /// Per-album gain (keeps relative loudness within an album).
    Album,
}

#[derive(Debug, Clone, Copy)]
struct ReplayGainConfig {
    mode: ReplayGainMode,
    preamp_db: f32,
    prevent_clipping: bool,
}

impl Default for ReplayGainConfig {
    fn default() -> Self {
        ReplayGainConfig {
            mode: ReplayGainMode::Off,
            preamp_db: 0.0,
            prevent_clipping: true,
        }
    }
}

/// Number of parametric-EQ bands, matching Rockbox's `EQ_NUM_BANDS`.
pub const EQ_BANDS: usize = rockbox_dsp::EQ_NUM_BANDS;

/// Pitch/speed ratio for normal playback (10000 = 100 %). Values are a
/// percentage ×100, so 10500 is +5 %; pitch and tempo shift together.
pub const PITCH_NORMAL: i32 = rockbox_dsp::PITCH_SPEED_100;

/// One parametric-EQ band. Band 0 is a low shelf, band `EQ_BANDS - 1` a
/// high shelf, and the bands in between are peaking filters. Units are
/// plain: `cutoff_hz` in Hz, `q` a Q factor, `gain_db` in dB.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EqBand {
    pub cutoff_hz: i32,
    pub q: f32,
    pub gain_db: f32,
}

/// The 10-band parametric equalizer.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Equalizer {
    /// Master enable for the whole EQ stage.
    pub enabled: bool,
    /// Pre-gain applied ahead of the bands to avoid clipping, in dB of
    /// headroom (positive value attenuates).
    pub precut_db: f32,
    /// Per-band settings; up to [`EQ_BANDS`] entries (extras are ignored).
    pub bands: Vec<EqBand>,
}

/// Standard center frequency (Hz) of each EQ band — octave-spaced, one per
/// [`EQ_BANDS`]. Used to build the [`EqPreset`] equalizers.
pub const EQ_BAND_FREQUENCIES: [i32; EQ_BANDS] =
    [32, 64, 125, 250, 500, 1_000, 2_000, 4_000, 8_000, 16_000];

/// Built-in, ready-to-use equalizer presets. Turn one into a live
/// [`Equalizer`] with [`EqPreset::equalizer`], or apply it directly with
/// [`Player::set_eq_preset`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EqPreset {
    Flat,
    Acoustic,
    BassBoost,
    BassReducer,
    Classical,
    Dance,
    Deep,
    Electronic,
    HipHop,
    Jazz,
    Latin,
    Loudness,
    Lounge,
    Piano,
    Pop,
    RnB,
    Rock,
    SmallSpeakers,
    TrebleBoost,
    TrebleReducer,
    VocalBoost,
}

impl EqPreset {
    /// Every preset, in a sensible display order (Flat first).
    pub const ALL: [EqPreset; 21] = [
        EqPreset::Flat,
        EqPreset::Acoustic,
        EqPreset::BassBoost,
        EqPreset::BassReducer,
        EqPreset::Classical,
        EqPreset::Dance,
        EqPreset::Deep,
        EqPreset::Electronic,
        EqPreset::HipHop,
        EqPreset::Jazz,
        EqPreset::Latin,
        EqPreset::Loudness,
        EqPreset::Lounge,
        EqPreset::Piano,
        EqPreset::Pop,
        EqPreset::RnB,
        EqPreset::Rock,
        EqPreset::SmallSpeakers,
        EqPreset::TrebleBoost,
        EqPreset::TrebleReducer,
        EqPreset::VocalBoost,
    ];

    /// A human-readable name, e.g. for a UI picker.
    pub fn name(self) -> &'static str {
        match self {
            EqPreset::Flat => "Flat",
            EqPreset::Acoustic => "Acoustic",
            EqPreset::BassBoost => "Bass Boost",
            EqPreset::BassReducer => "Bass Reducer",
            EqPreset::Classical => "Classical",
            EqPreset::Dance => "Dance",
            EqPreset::Deep => "Deep",
            EqPreset::Electronic => "Electronic",
            EqPreset::HipHop => "Hip-Hop",
            EqPreset::Jazz => "Jazz",
            EqPreset::Latin => "Latin",
            EqPreset::Loudness => "Loudness",
            EqPreset::Lounge => "Lounge",
            EqPreset::Piano => "Piano",
            EqPreset::Pop => "Pop",
            EqPreset::RnB => "R&B",
            EqPreset::Rock => "Rock",
            EqPreset::SmallSpeakers => "Small Speakers",
            EqPreset::TrebleBoost => "Treble Boost",
            EqPreset::TrebleReducer => "Treble Reducer",
            EqPreset::VocalBoost => "Vocal Boost",
        }
    }

    /// Per-band gains in dB. Index `i` corresponds to
    /// `EQ_BAND_FREQUENCIES[i]` (32 Hz … 16 kHz).
    pub fn gains(self) -> [f32; EQ_BANDS] {
        // Bands:      32   64  125  250  500   1k   2k   4k   8k  16k
        match self {
            EqPreset::Flat => [0.0; EQ_BANDS],
            EqPreset::Acoustic => [5.0, 5.0, 4.0, 1.0, 2.0, 2.0, 3.0, 4.0, 3.0, 2.0],
            EqPreset::BassBoost => [7.0, 6.0, 5.0, 3.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            EqPreset::BassReducer => [-6.0, -5.0, -4.0, -2.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            EqPreset::Classical => [5.0, 4.0, 3.0, 2.0, -1.0, -1.0, 0.0, 2.0, 3.0, 4.0],
            EqPreset::Dance => [4.0, 7.0, 5.0, 0.0, 2.0, 3.0, 5.0, 4.0, 3.0, 0.0],
            EqPreset::Deep => [5.0, 4.0, 2.0, 1.0, 3.0, 2.0, 1.0, -2.0, -4.0, -5.0],
            EqPreset::Electronic => [4.0, 4.0, 1.0, 0.0, -2.0, 2.0, 1.0, 1.0, 4.0, 5.0],
            EqPreset::HipHop => [5.0, 4.0, 2.0, 3.0, -1.0, -1.0, 1.0, -1.0, 2.0, 3.0],
            EqPreset::Jazz => [4.0, 3.0, 1.0, 2.0, -2.0, -2.0, 0.0, 1.0, 3.0, 4.0],
            EqPreset::Latin => [4.0, 3.0, 0.0, 0.0, -2.0, -2.0, -2.0, 0.0, 3.0, 5.0],
            EqPreset::Loudness => [6.0, 4.0, 0.0, 0.0, -3.0, 0.0, -1.0, -5.0, 5.0, 1.0],
            EqPreset::Lounge => [-3.0, -1.0, -1.0, 1.0, 4.0, 2.0, 0.0, -2.0, 2.0, 1.0],
            EqPreset::Piano => [3.0, 2.0, 0.0, 2.0, 3.0, 1.0, 3.0, 4.0, 3.0, 3.0],
            EqPreset::Pop => [-2.0, -1.0, 0.0, 2.0, 4.0, 4.0, 2.0, 0.0, -1.0, -2.0],
            EqPreset::RnB => [3.0, 7.0, 6.0, 1.0, -2.0, -1.0, 2.0, 3.0, 3.0, 4.0],
            EqPreset::Rock => [5.0, 4.0, 3.0, 1.0, -1.0, -1.0, 1.0, 3.0, 4.0, 5.0],
            EqPreset::SmallSpeakers => [6.0, 5.0, 4.0, 2.0, 1.0, 0.0, -1.0, -2.0, -3.0, -4.0],
            EqPreset::TrebleBoost => [0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 3.0, 5.0, 6.0, 7.0],
            EqPreset::TrebleReducer => [0.0, 0.0, 0.0, 0.0, 0.0, -1.0, -3.0, -4.0, -5.0, -6.0],
            EqPreset::VocalBoost => [-2.0, -3.0, -3.0, 1.0, 4.0, 4.0, 3.0, 1.0, 0.0, -2.0],
        }
    }

    /// Build a ready-to-use [`Equalizer`]: enabled, all ten bands set at the
    /// standard center frequencies with Q 1.0, and a precut equal to the
    /// largest positive band gain so boosted presets don't clip. `Flat`
    /// yields an enabled but transparent EQ.
    pub fn equalizer(self) -> Equalizer {
        let gains = self.gains();
        let precut_db = gains.iter().cloned().fold(0.0_f32, f32::max);
        let bands = EQ_BAND_FREQUENCIES
            .iter()
            .zip(gains)
            .map(|(&cutoff_hz, gain_db)| EqBand {
                cutoff_hz,
                q: 1.0,
                gain_db,
            })
            .collect();
        Equalizer {
            enabled: true,
            precut_db,
            bands,
        }
    }
}

/// Bass/treble shelving tone controls (0/0 dB disables the stage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ToneControls {
    pub bass_db: i32,
    pub treble_db: i32,
    /// Bass shelf cutoff in Hz; 0 keeps the default (200 Hz).
    pub bass_cutoff_hz: i32,
    /// Treble shelf cutoff in Hz; 0 keeps the default (3.5 kHz).
    pub treble_cutoff_hz: i32,
}

/// Haas-effect surround widening.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Surround {
    /// Haas delay in ms; 0 disables the stage.
    pub delay_ms: i32,
    /// Left/right balance in percent.
    pub balance: i32,
    /// Low band-split cutoff in Hz; 0/0 keeps the defaults.
    pub cutoff_low_hz: i32,
    /// High band-split cutoff in Hz.
    pub cutoff_high_hz: i32,
}

/// Channel-mixing mode (`SOUND_CHAN_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChannelMode {
    #[default]
    Stereo,
    Mono,
    /// Custom stereo width — see [`Player::set_stereo_width`].
    Custom,
    MonoLeft,
    MonoRight,
    Karaoke,
    /// Swap the left and right channels.
    Swap,
}

impl ChannelMode {
    fn to_raw(self) -> i32 {
        match self {
            ChannelMode::Stereo => rockbox_dsp::SOUND_CHAN_STEREO,
            ChannelMode::Mono => rockbox_dsp::SOUND_CHAN_MONO,
            ChannelMode::Custom => rockbox_dsp::SOUND_CHAN_CUSTOM,
            ChannelMode::MonoLeft => rockbox_dsp::SOUND_CHAN_MONO_LEFT,
            ChannelMode::MonoRight => rockbox_dsp::SOUND_CHAN_MONO_RIGHT,
            ChannelMode::Karaoke => rockbox_dsp::SOUND_CHAN_KARAOKE,
            ChannelMode::Swap => rockbox_dsp::SOUND_CHAN_SWAP,
        }
    }
}

/// Dynamic-range compressor. `threshold_db == 0` disables the stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Compressor {
    /// Threshold in dB below full scale (e.g. -20); 0 = off.
    pub threshold_db: i32,
    /// Make-up gain: 0 = off, 1 = auto.
    pub makeup_gain: i32,
    /// Ratio index: 0 = 2:1, 1 = 4:1, 2 = 6:1, 3 = 10:1, 4 = limit.
    pub ratio: i32,
    /// Knee index: 0 = hard, 1 = medium, 2 = soft.
    pub knee: i32,
    /// Attack time in ms.
    pub attack_ms: i32,
    /// Release time in ms.
    pub release_ms: i32,
}

impl Compressor {
    fn to_raw(self) -> rockbox_dsp::compressor_settings {
        rockbox_dsp::compressor_settings {
            threshold: self.threshold_db,
            makeup_gain: self.makeup_gain,
            ratio: self.ratio,
            knee: self.knee,
            release_time: self.release_ms,
            attack_time: self.attack_ms,
        }
    }
}

/// Headphone **crossfeed** mode (`crossfeed_type`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossfeedMode {
    /// Crossfeed disabled.
    #[default]
    Off,
    /// Meier crossfeed — a fixed, natural-sounding profile.
    Meier,
    /// Custom crossfeed, driven by the [`Crossfeed`] gain/cutoff fields.
    Custom,
}

impl CrossfeedMode {
    fn to_raw(self) -> i32 {
        match self {
            CrossfeedMode::Off => rockbox_dsp::CROSSFEED_OFF,
            CrossfeedMode::Meier => rockbox_dsp::CROSSFEED_MEIER,
            CrossfeedMode::Custom => rockbox_dsp::CROSSFEED_CUSTOM,
        }
    }
}

/// Headphone crossfeed — bleeds some of each channel into the other to ease
/// the hard L/R separation of headphones. The `*_cross_*` fields only apply
/// in [`CrossfeedMode::Custom`]. All gains are in **tenths of a dB** (≤ 0).
/// Defaults mirror Rockbox's own.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Crossfeed {
    pub mode: CrossfeedMode,
    /// Dry-mix (direct) gain in tenths of a dB (≤ 0; -15 = −1.5 dB).
    pub direct_gain: i32,
    /// Custom cross-mix low-frequency gain in tenths of a dB (≤ 0).
    pub cross_gain: i32,
    /// Custom cross-mix high-frequency attenuation in tenths of a dB (≤ 0).
    pub high_freq_gain: i32,
    /// Custom cross-mix high-frequency cutoff in Hz.
    pub high_freq_cutoff: i32,
}

impl Default for Crossfeed {
    fn default() -> Self {
        Crossfeed {
            mode: CrossfeedMode::Off,
            direct_gain: -15,
            cross_gain: -60,
            high_freq_gain: -160,
            high_freq_cutoff: 700,
        }
    }
}

/// **Perceptual Bass Enhancement** (PBE). `strength == 0` disables the stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BassEnhancement {
    /// Strength as a percent (0 = off … 100).
    pub strength: i32,
    /// Pre-cut headroom in tenths of a dB (≤ 0), applied ahead of the boost.
    pub precut: i32,
}

/// Initial configuration for the full DSP chain (everything past
/// ReplayGain). Every stage defaults to neutral, so
/// `DspSettings::default()` is a transparent pipeline; each stage can also
/// be changed live through the matching [`Player`] setter.
#[derive(Debug, Clone)]
pub struct DspSettings {
    pub equalizer: Equalizer,
    pub tone: ToneControls,
    /// Headphone crossfeed.
    pub crossfeed: Crossfeed,
    pub surround: Surround,
    pub channel_mode: ChannelMode,
    /// Custom stereo width in percent (100 = unchanged); only audible with
    /// [`ChannelMode::Custom`].
    pub stereo_width: i32,
    /// Perceptual Bass Enhancement.
    pub bass_enhancement: BassEnhancement,
    /// Auditory Fatigue Reduction level: 0 off, 1 weak, 2 moderate, 3 strong.
    pub fatigue_reduction: i32,
    pub compressor: Compressor,
    /// Output dithering + noise shaping.
    pub dither: bool,
    /// Pitch/speed ratio ([`PITCH_NORMAL`] = normal); pitch and tempo shift
    /// together.
    pub pitch: i32,
}

impl Default for DspSettings {
    fn default() -> Self {
        DspSettings {
            equalizer: Equalizer::default(),
            tone: ToneControls::default(),
            crossfeed: Crossfeed::default(),
            surround: Surround::default(),
            channel_mode: ChannelMode::default(),
            stereo_width: 100,
            bass_enhancement: BassEnhancement::default(),
            fatigue_reduction: 0,
            compressor: Compressor::default(),
            dither: false,
            pitch: PITCH_NORMAL,
        }
    }
}

/// Whether the player is stopped, playing or paused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

/// Where to insert tracks into the queue, mirroring Rockbox's
/// `playlist_insert_track` position constants (`apps/playlist.h`).
///
/// Positions are resolved relative to the currently-playing track: the
/// engine shifts its play index as needed so the current track keeps
/// playing when tracks are inserted before or at it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertPosition {
    /// Add at the very beginning of the queue (`PLAYLIST_PREPEND`).
    Prepend,
    /// Add after the last inserted track, or immediately after the current
    /// track if none were inserted since (`PLAYLIST_INSERT`). Successive
    /// `Insert`s therefore keep their relative order, growing a block right
    /// after the current track.
    Insert,
    /// Add immediately after the current track — "play next"
    /// (`PLAYLIST_INSERT_FIRST`).
    InsertNext,
    /// Add to the end of the queue — "play last" (`PLAYLIST_INSERT_LAST`).
    InsertLast,
    /// Add at a random point between the current track and the end of the
    /// queue (`PLAYLIST_INSERT_SHUFFLED`). When stopped (not started), the
    /// random point spans the whole queue.
    InsertShuffled,
    /// Add at a random point within the region appended by this call,
    /// leaving all earlier tracks in place (`PLAYLIST_INSERT_LAST_SHUFFLED`).
    /// With a multi-track insert this shuffles the new tracks among
    /// themselves at the tail of the queue.
    InsertLastShuffled,
    /// Erase the queue and cue the new tracks from the start
    /// (`PLAYLIST_REPLACE`). If playing, playback continues into the first
    /// new track.
    Replace,
    /// Insert at an explicit index (clamped to the queue length).
    Index(usize),
}

const ST_STOPPED: u8 = 0;
const ST_PLAYING: u8 = 1;
const ST_PAUSED: u8 = 2;

/// Output levels for a meter, as produced by the audio callback.
///
/// Each value is a 0..1 RMS over one callback buffer — roughly 10-20 ms —
/// taken *after* volume, balance and fade, so it reflects what is actually
/// leaving the device rather than what was decoded.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Levels {
    pub left: f32,
    pub right: f32,
    /// The same signal through a one-pole low-pass at roughly 200 Hz. A meter
    /// driven by this moves with the bass, which is what reads as "following
    /// the music"; full-band RMS is dominated by whatever is loudest and
    /// tends to sit near the top.
    pub low_left: f32,
    pub low_right: f32,
    /// A coarse spectrum: RMS per band, low to high, over the same buffer.
    /// Bands come from a ladder of one-pole low-passes at log-spaced cutoffs
    /// (the difference of two neighbours is a band-pass) — trivial per-sample
    /// arithmetic, safe inside the audio callback. What a spectrum visualiser
    /// draws; the two scalar pairs above stay for plain meters.
    pub bands: [f32; SPECTRUM_BANDS],
}

/// How many spectrum bands [`Levels::bands`] carries.
pub const SPECTRUM_BANDS: usize = 16;

/// Ladder cutoffs, Hz — log-spaced ~28 Hz → 16 kHz, one per band edge.
const BAND_CUTOFFS: [f32; SPECTRUM_BANDS] = [
    40.0, 60.0, 90.0, 135.0, 200.0, 300.0, 450.0, 675.0, 1_000.0, 1_500.0, 2_250.0, 3_400.0,
    5_000.0, 7_500.0, 11_000.0, 16_000.0,
];

/// A snapshot of the player's status.
#[derive(Debug, Clone)]
pub struct Status {
    pub state: PlaybackState,
    /// Index of the current track in the queue, if any.
    pub index: Option<usize>,
    /// Playback position within the current track (accounts for the
    /// decode-ahead buffer, so it tracks what you actually hear).
    pub position: Duration,
    /// Length of the current track.
    pub duration: Duration,
    /// Metadata of the current track, if one is loaded.
    pub metadata: Option<Metadata>,
    /// Number of tracks in the queue.
    pub queue_len: usize,
    /// Whether shuffle playback is enabled.
    pub shuffle: bool,
    /// The current repeat mode.
    pub repeat: RepeatMode,
    /// Output levels at the moment of the call, for a meter. Zero when
    /// nothing is playing.
    pub levels: Levels,
}

/// Configuration for [`Player::with_config`].
pub struct PlayerConfig {
    /// Where audio is sent: the system device via `cpal` (default), or a
    /// raw **S16LE** stereo stream to stdout / a FIFO / a Unix or TCP
    /// socket. See [`OutputConfig`].
    pub output: OutputConfig,
    /// Output sample rate. `None` uses the output device's default; every
    /// track is resampled to this rate by the DSP so mixed-rate queues
    /// work. For the non-`cpal` byte-stream backends there is no device to
    /// query, so `None` falls back to 44100 Hz.
    pub sample_rate: Option<u32>,
    /// Seconds of audio to decode ahead into the ring buffer.
    pub buffer_seconds: f32,
    pub crossfade: CrossfadeSettings,
    pub replaygain_mode: ReplayGainMode,
    pub replaygain_preamp_db: f32,
    pub replaygain_prevent_clipping: bool,
    /// Initial state of the full DSP chain (EQ, tone, surround, channel
    /// mixing, compressor, dither, pitch). Defaults to a transparent
    /// pipeline.
    pub dsp: DspSettings,
    /// Whether shuffle playback starts enabled.
    pub shuffle: bool,
    /// Initial repeat mode.
    pub repeat: RepeatMode,
    /// Initial volume, 0.0..=1.0.
    pub volume: f32,
    /// When set, the queue and the exact playback position are auto-saved to
    /// this file (an extended `.m3u8`) as playback proceeds, and
    /// [`Player::resume`] restores them. `None` disables persistence.
    pub resume_file: Option<PathBuf>,
    /// How often the resume file is refreshed while playing (in addition to
    /// immediate saves on pause / stop / track change / shutdown). Only used
    /// when `resume_file` is set.
    pub resume_save_interval: Duration,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        PlayerConfig {
            output: OutputConfig::default(),
            sample_rate: None,
            buffer_seconds: 4.0,
            crossfade: CrossfadeSettings::default(),
            replaygain_mode: ReplayGainMode::Off,
            replaygain_preamp_db: 0.0,
            replaygain_prevent_clipping: true,
            dsp: DspSettings::default(),
            shuffle: false,
            repeat: RepeatMode::Off,
            volume: 1.0,
            resume_file: None,
            resume_save_interval: Duration::from_secs(5),
        }
    }
}

impl PlayerConfig {
    /// Start a fluent [`PlayerConfigBuilder`] seeded with the defaults:
    ///
    /// ```no_run
    /// use rockbox_playback::{PlayerConfig, RepeatMode};
    ///
    /// let player = PlayerConfig::builder()
    ///     .volume(0.8)
    ///     .shuffle(true)
    ///     .repeat(RepeatMode::All)
    ///     .open()?;                     // build + construct in one step
    /// # Ok::<(), rockbox_playback::Error>(())
    /// ```
    pub fn builder() -> PlayerConfigBuilder {
        PlayerConfigBuilder {
            cfg: PlayerConfig::default(),
        }
    }
}

/// Fluent builder for [`PlayerConfig`] — obtain one from [`PlayerConfig::builder`].
/// Every method takes and returns `self` so calls chain; finish with
/// [`build`](PlayerConfigBuilder::build) (get the config) or
/// [`open`](PlayerConfigBuilder::open) (build it *and* construct the player).
pub struct PlayerConfigBuilder {
    cfg: PlayerConfig,
}

impl PlayerConfigBuilder {
    /// Where audio is sent (system device, stdout, FIFO, Unix or TCP
    /// socket). See [`OutputConfig`]; defaults to [`OutputConfig::Cpal`].
    pub fn output(mut self, output: OutputConfig) -> Self {
        self.cfg.output = output;
        self
    }
    /// Output sample rate in Hz. Unset (the default) uses the output device's
    /// native rate; every track is resampled to this rate.
    pub fn sample_rate(mut self, hz: u32) -> Self {
        self.cfg.sample_rate = Some(hz);
        self
    }
    /// Seconds of audio to decode ahead into the ring buffer.
    pub fn buffer_seconds(mut self, secs: f32) -> Self {
        self.cfg.buffer_seconds = secs;
        self
    }
    /// Crossfade behaviour (see [`CrossfadeSettings`]).
    pub fn crossfade(mut self, crossfade: CrossfadeSettings) -> Self {
        self.cfg.crossfade = crossfade;
        self
    }
    /// ReplayGain mode, preamp in dB, and clip prevention — mirrors
    /// [`Player::set_replaygain`].
    pub fn replaygain(
        mut self,
        mode: ReplayGainMode,
        preamp_db: f32,
        prevent_clipping: bool,
    ) -> Self {
        self.cfg.replaygain_mode = mode;
        self.cfg.replaygain_preamp_db = preamp_db;
        self.cfg.replaygain_prevent_clipping = prevent_clipping;
        self
    }
    /// Initial DSP-chain settings (EQ, tone, surround, …). See [`DspSettings`].
    pub fn dsp(mut self, dsp: DspSettings) -> Self {
        self.cfg.dsp = dsp;
        self
    }
    /// Whether shuffle playback starts enabled.
    pub fn shuffle(mut self, enabled: bool) -> Self {
        self.cfg.shuffle = enabled;
        self
    }
    /// Initial [`RepeatMode`].
    pub fn repeat(mut self, mode: RepeatMode) -> Self {
        self.cfg.repeat = mode;
        self
    }
    /// Initial volume, 0.0..=1.0.
    pub fn volume(mut self, volume: f32) -> Self {
        self.cfg.volume = volume;
        self
    }
    /// Enable resume: auto-persist the queue + exact position to this `.m3u8`.
    pub fn resume_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.cfg.resume_file = Some(path.into());
        self
    }
    /// How often the resume file is refreshed while playing.
    pub fn resume_save_interval(mut self, interval: Duration) -> Self {
        self.cfg.resume_save_interval = interval;
        self
    }
    /// Finish building and return the [`PlayerConfig`].
    pub fn build(self) -> PlayerConfig {
        self.cfg
    }
    /// Build the config and construct the [`Player`] in one step (a shortcut
    /// for `Player::with_config(builder.build())`).
    pub fn open(self) -> Result<Player, Error> {
        Player::with_config(self.cfg)
    }
}

enum Command {
    OpenStream(managed::StreamInput),
    SetQueue(Vec<PathBuf>),
    Enqueue(PathBuf),
    /// Insert one or more tracks at a Rockbox insertion position.
    Insert(Vec<PathBuf>, InsertPosition),
    /// Remove the track at the given queue index (0-based).
    Remove(usize),
    /// Empty the queue and stop playback.
    Clear,
    /// Restore a persisted queue + exact position (does not auto-play).
    Resume(ResumeState),
    /// Force an immediate resume-file save.
    SaveResume,
    Play,
    Pause,
    Toggle,
    Stop,
    Next,
    Previous,
    SkipTo(usize),
    Seek(Duration),
    SetVolume(f32),
    SetBalance(i32),
    SetCrossfade(CrossfadeSettings),
    SetReplayGain(ReplayGainConfig),
    SetEqEnabled(bool),
    SetEqBand(usize, EqBand),
    SetEqPrecut(f32),
    SetEqualizer(Equalizer),
    SetTone(ToneControls),
    SetBass(i32),
    SetTreble(i32),
    SetBassCutoff(i32),
    SetTrebleCutoff(i32),
    SetCrossfeed(Crossfeed),
    SetSurround(Surround),
    SetChannelMode(ChannelMode),
    SetStereoWidth(i32),
    SetBassEnhancement(BassEnhancement),
    SetFatigueReduction(i32),
    SetCompressor(Compressor),
    SetDither(bool),
    SetPitch(i32),
    SetShuffle(bool),
    SetRepeat(RepeatMode),
    Shutdown,
}

/// State shared between the public [`Player`] handle, the engine thread
/// and the cpal callback.
struct Shared {
    state: AtomicU8,
    /// Decode position of the current track in ms (ahead of playback by
    /// the ring buffer's fill).
    decode_pos_ms: AtomicU64,
    duration_ms: AtomicU64,
    index: AtomicUsize, // usize::MAX = none
    queue_len: AtomicUsize,
    /// Target output amplitude the callback ramps toward (f32 bits):
    /// `volume` when playing, 0 when paused/stopped.
    target_amp: AtomicU32,
    /// User volume 0.0..=1.0 (f32 bits).
    volume: AtomicU32,
    /// Stereo balance, -100 (full left) ..= +100 (full right); 0 = centred.
    balance: AtomicI32,
    /// Per-channel gains derived from `balance` (f32 bits) so the output
    /// callback can apply the pan without recomputing it per frame.
    balance_gain_l: AtomicU32,
    balance_gain_r: AtomicU32,
    output_rate: AtomicU32,
    /// Whether shuffle playback is enabled (mirrored from the engine).
    shuffle: AtomicBool,
    /// The current [`RepeatMode`] as `u8` (mirrored from the engine).
    repeat: AtomicU8,
    ring: Mutex<VecDeque<i16>>,
    /// Output levels, for a meter. Four `f32`s bit-packed into atomics so the
    /// audio callback never takes a lock to publish them — a meter is not
    /// worth a priority inversion in the callback that feeds the speakers.
    ///
    /// Each is a 0..1 RMS over the callback's own buffer: `low_*` is the same
    /// signal through a one-pole low-pass, which is what makes a meter move
    /// with the bass rather than with sibilance.
    level_l: AtomicU32,
    level_r: AtomicU32,
    level_low_l: AtomicU32,
    level_low_r: AtomicU32,
    /// One atomic per spectrum band; written by the callback, read anywhere.
    level_bands: [AtomicU32; SPECTRUM_BANDS],
    meta: Mutex<Option<Metadata>>,
    /// Live mirror of the DSP-chain settings, updated by the engine as it
    /// applies each `Set*` command, so [`Player::dsp_settings`] can read the
    /// current state without a round-trip to the engine thread.
    dsp: Mutex<DspSettings>,
    /// Mirror of the engine's queue so the handle can read it (for
    /// `queue()` / `export_m3u`) without a round-trip to the engine thread.
    queue: Mutex<Vec<PathBuf>>,
    managed: Mutex<Option<StreamSession>>,
    next_generation: AtomicU64,
    // ring also gates revocation and final nonblocking writes.
    output_generation: AtomicU64,
    output_session: Mutex<Option<StreamSession>>,
    output_boundary: StreamOutputBoundary,
    output_epoch: AtomicU64,
    output_nonblocking: bool,
    output_backpressure: AtomicU64,
    output_bytes: AtomicU64,
    output_nonzero_bytes: AtomicU64,
    output_failed: AtomicBool,
    writer_exited: AtomicBool,
}

impl Shared {
    fn revoke_output(&self, generation: u64) {
        let mut ring = self.ring.lock().unwrap();
        if self.output_generation.load(Ordering::Relaxed) == generation {
            self.output_generation.store(0, Ordering::Relaxed);
            self.output_session.lock().unwrap().take();
            self.output_epoch.fetch_add(1, Ordering::Relaxed);
            self.target_amp.store(0f32.to_bits(), Ordering::Relaxed);
            ring.clear();
        }
    }

    // Called under the ring/output gate, after actual output acceptance.
    fn record_stream_output(&self, frames: usize) {
        if frames > 0 {
            if let Some(session) = self.output_session.lock().unwrap().as_ref() {
                session.record_output(frames);
            }
        }
    }

    fn set_volume_bits(&self, v: f32) {
        self.volume
            .store(v.clamp(0.0, 1.0).to_bits(), Ordering::Relaxed);
    }
    fn volume_f32(&self) -> f32 {
        f32::from_bits(self.volume.load(Ordering::Relaxed))
    }
    /// Store the balance and the derived per-channel gains. A linear pan:
    /// panning right (`b > 0`) attenuates the left channel and vice-versa,
    /// so the un-panned channel always stays at unity.
    fn set_balance_value(&self, balance: i32) {
        let b = balance.clamp(-100, 100);
        self.balance.store(b, Ordering::Relaxed);
        let (gl, gr) = if b >= 0 {
            (1.0 - b as f32 / 100.0, 1.0)
        } else {
            (1.0, 1.0 + b as f32 / 100.0)
        };
        self.balance_gain_l.store(gl.to_bits(), Ordering::Relaxed);
        self.balance_gain_r.store(gr.to_bits(), Ordering::Relaxed);
    }
    fn balance_value(&self) -> i32 {
        self.balance.load(Ordering::Relaxed)
    }
    fn ring_frames(&self) -> usize {
        self.ring.lock().unwrap().len() / 2
    }

    /// The levels the audio callback last published.
    fn levels(&self) -> Levels {
        Levels {
            left: f32::from_bits(self.level_l.load(Ordering::Relaxed)),
            right: f32::from_bits(self.level_r.load(Ordering::Relaxed)),
            low_left: f32::from_bits(self.level_low_l.load(Ordering::Relaxed)),
            low_right: f32::from_bits(self.level_low_r.load(Ordering::Relaxed)),
            bands: std::array::from_fn(|i| {
                f32::from_bits(self.level_bands[i].load(Ordering::Relaxed))
            }),
        }
    }

    /// Publish one buffer's levels. Called from the audio callback, so it
    /// takes no locks and allocates nothing.
    fn set_levels(&self, levels: Levels) {
        self.level_l.store(levels.left.to_bits(), Ordering::Relaxed);
        self.level_r
            .store(levels.right.to_bits(), Ordering::Relaxed);
        self.level_low_l
            .store(levels.low_left.to_bits(), Ordering::Relaxed);
        self.level_low_r
            .store(levels.low_right.to_bits(), Ordering::Relaxed);
        for (slot, band) in self.level_bands.iter().zip(levels.bands) {
            slot.store(band.to_bits(), Ordering::Relaxed);
        }
    }
}

/// Errors constructing a [`Player`].
#[derive(Debug)]
pub enum Error {
    /// No output audio device available (`cpal` backend).
    NoOutputDevice,
    /// cpal could not build/start the stream, or a byte-stream backend
    /// (stdout / FIFO / Unix / TCP) failed to open or connect.
    Stream(String),
    /// The requested [`OutputConfig`] backend was compiled out (e.g. the
    /// `cpal` backend without the `cpal` feature, or a FIFO/Unix socket on
    /// a non-Unix platform).
    UnsupportedBackend(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoOutputDevice => write!(f, "no output audio device"),
            Error::Stream(e) => write!(f, "audio stream error: {e}"),
            Error::UnsupportedBackend(e) => write!(f, "unsupported output backend: {e}"),
        }
    }
}

impl std::error::Error for Error {}

/// The live output backend, kept on the [`Player`] handle so output stays
/// alive for the player's lifetime. `cpal` streams are non-`Send`, which is
/// why the handle (not the engine thread) owns them. The variants are held
/// purely for their `Drop` side effects (stop the stream / join the writer).
#[allow(dead_code)]
enum OutputHandle {
    /// System audio device: a `cpal` stream whose callback drains the ring.
    #[cfg(feature = "cpal")]
    Cpal(cpal::Stream),
    /// A byte-stream sink (stdout / FIFO / Unix / TCP): a writer thread
    /// drains the ring, converts to S16LE and paces to real time. Dropping
    /// this signals the thread to stop and joins it.
    Stream(StreamSink),
}

/// Owns the writer thread for a byte-stream backend and stops it on drop.
struct StreamSink {
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for StreamSink {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// Byte-stream writer observations, not an audible-consumption clock.
/// Counters cover the writer lifetime, including silence and alignment padding.
#[derive(Debug, Clone, Copy)]
pub struct OutputSnapshot {
    pub nonblocking: bool,
    pub generation: u64,
    pub epoch: u64,
    pub backpressure_events: u64,
    pub bytes_written: u64,
    pub nonzero_bytes_written: u64,
    pub output_failed: bool,
    pub writer_exited: bool,
}

/// The player handle. Cloneable-free but `Send` controls are issued
/// through it; the output backend lives here and keeps output alive for the
/// player's lifetime.
pub struct Player {
    tx: Sender<Command>,
    shared: Arc<Shared>,
    _output: OutputHandle,
    engine: Option<std::thread::JoinHandle<()>>,
    /// Resume file path (mirrors `PlayerConfig::resume_file`) so
    /// [`Player::resume`] can read it on the handle side.
    resume_file: Option<PathBuf>,
}

impl Player {
    /// Create a player on the default output device with default settings.
    pub fn new() -> Result<Self, Error> {
        Self::with_config(PlayerConfig::default())
    }

    /// Create a player with explicit configuration.
    ///
    /// The output backend is chosen by [`PlayerConfig::output`]. Note that a
    /// *listening* socket backend ([`SocketMode::Listen`]) blocks here until
    /// a client connects, and a *connecting* one requires the receiver to be
    /// up already.
    pub fn with_config(config: PlayerConfig) -> Result<Self, Error> {
        #[cfg(feature = "cpal")]
        if config.output == OutputConfig::Cpal {
            let host = cpal::default_host();
            let device = host.default_output_device().ok_or(Error::NoOutputDevice)?;
            let default_cfg = device
                .default_output_config()
                .map_err(|e| Error::Stream(e.to_string()))?;
            let rate = config
                .sample_rate
                .unwrap_or_else(|| default_cfg.sample_rate().0);
            let shared = make_shared(&config, rate);
            let stream = build_stream(&device, rate, Arc::clone(&shared))?;
            stream.play().map_err(|e| Error::Stream(e.to_string()))?;
            return Ok(assemble(config, rate, shared, OutputHandle::Cpal(stream)));
        }
        #[cfg(not(feature = "cpal"))]
        if config.output == OutputConfig::Cpal {
            return Err(Error::UnsupportedBackend(
                "cpal backend requires the `cpal` feature".into(),
            ));
        }

        // Byte-stream backends (stdout / FIFO / Unix / TCP): no device to
        // query, so an unset sample rate falls back to CD quality.
        let rate = config.sample_rate.unwrap_or(44100);
        let writer = config
            .output
            .open_writer()
            .map_err(|e| Error::Stream(e.to_string()))?;
        let shared = make_shared(&config, rate);
        let sink = spawn_stream_writer(writer, rate, Arc::clone(&shared));
        Ok(assemble(config, rate, shared, OutputHandle::Stream(sink)))
    }

    /// Start one forward-only, host-owned stream. No URL or credentials enter
    /// the persisted queue. The host must supply an idempotent, nonblocking
    /// wake callback that cancels every blocked read and joins on reader drop.
    /// Pause/stop/queue replacement cancel this session. Resume/seek require
    /// the host to resolve a new stream at its confirmed absolute checkpoint.
    /// EOF never auto-advances: completion requires evidence outside this API.
    /// Byte-stream cancellation requires a nonblocking FIFO/socket output.
    /// Legacy stdout and socket construction are outside that guarantee.
    /// Data already submitted to a device/receiver cannot be recalled here.
    pub fn play_stream(
        &self,
        reader: Box<dyn std::io::Read + Send>,
        format_ext: String,
        metadata: Metadata,
        wake: impl Fn() + Send + Sync + 'static,
    ) -> StreamSession {
        let mut active = self.shared.managed.lock().unwrap();
        if let Some(old) = active.as_ref() {
            old.cancel();
        }
        let generation = self.shared.next_generation.fetch_add(1, Ordering::Relaxed);
        let weak = Arc::downgrade(&self.shared);
        let input =
            managed::StreamInput::new(reader, format_ext, metadata, generation, wake, move || {
                if let Some(shared) = weak.upgrade() {
                    shared.revoke_output(generation);
                }
            });
        let session = input.session.clone();
        session.configure_output(self.sample_rate(), self.shared.output_boundary);
        {
            let mut ring = self.shared.ring.lock().unwrap();
            ring.clear();
            self.shared
                .target_amp
                .store(0f32.to_bits(), Ordering::Relaxed);
            self.shared
                .output_generation
                .store(generation, Ordering::Relaxed);
            *self.shared.output_session.lock().unwrap() = Some(session.clone());
            self.shared.output_epoch.fetch_add(1, Ordering::Relaxed);
        }
        *active = Some(session.clone());
        let _ = self.tx.send(Command::OpenStream(input));
        session
    }

    // Serialize invalidation with registration so a stop cannot miss a
    // queued-but-not-yet-opened stream. Never wait for network IO here.
    fn send_invalidating(&self, command: Command) {
        let mut active = self.shared.managed.lock().unwrap();
        if let Some(session) = active.take() {
            session.cancel();
        }
        let _ = self.tx.send(command);
    }

    /// Byte acceptance is not receiver consumption. CPAL is not counted here.
    pub fn output_snapshot(&self) -> OutputSnapshot {
        let _gate = self.shared.ring.lock().unwrap();
        OutputSnapshot {
            nonblocking: self.shared.output_nonblocking,
            generation: self.shared.output_generation.load(Ordering::Relaxed),
            epoch: self.shared.output_epoch.load(Ordering::Relaxed),
            backpressure_events: self.shared.output_backpressure.load(Ordering::Relaxed),
            bytes_written: self.shared.output_bytes.load(Ordering::Relaxed),
            nonzero_bytes_written: self.shared.output_nonzero_bytes.load(Ordering::Relaxed),
            output_failed: self.shared.output_failed.load(Ordering::Relaxed),
            writer_exited: self.shared.writer_exited.load(Ordering::Relaxed),
        }
    }

    /// The output sample rate everything is resampled to.
    pub fn sample_rate(&self) -> u32 {
        self.shared.output_rate.load(Ordering::Relaxed)
    }

    /// Replace the queue. Each entry may be a **local file path**, an
    /// **`http(s)://` URL** to a finite remote file (streamed on demand via
    /// range requests), or a **live-radio / streaming URL** (decoded on the
    /// fly, never fully downloaded). Does not change playback state; call
    /// [`Player::play`] to start.
    pub fn set_queue<I, P>(&self, tracks: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        let v: Vec<PathBuf> = tracks.into_iter().map(Into::into).collect();
        self.send_invalidating(Command::SetQueue(v));
    }

    /// Append one track to the end of the queue. The track may be a **local
    /// file path**, an **`http(s)://` URL** to a finite remote file, or a
    /// **live-radio / streaming URL**.
    pub fn enqueue(&self, track: impl Into<PathBuf>) {
        let _ = self.tx.send(Command::Enqueue(track.into()));
    }

    /// Insert one track at a Rockbox insertion position (see
    /// [`InsertPosition`]). Does not change playback state.
    pub fn insert(&self, track: impl Into<PathBuf>, position: InsertPosition) {
        let _ = self.tx.send(Command::Insert(vec![track.into()], position));
    }

    /// Insert several tracks at a Rockbox insertion position, preserving
    /// their order (except for the shuffled positions). See
    /// [`InsertPosition`].
    pub fn insert_tracks<I, P>(&self, tracks: I, position: InsertPosition)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        let v: Vec<PathBuf> = tracks.into_iter().map(Into::into).collect();
        let _ = self.tx.send(Command::Insert(v, position));
    }

    /// "Play next" — insert a track immediately after the current one
    /// ([`InsertPosition::InsertNext`]).
    pub fn insert_next(&self, track: impl Into<PathBuf>) {
        self.insert(track, InsertPosition::InsertNext);
    }

    /// "Play last" — append a track to the end of the queue
    /// ([`InsertPosition::InsertLast`]).
    pub fn insert_last(&self, track: impl Into<PathBuf>) {
        self.insert(track, InsertPosition::InsertLast);
    }

    /// Insert a track at a random point between the current track and the
    /// end ([`InsertPosition::InsertShuffled`]).
    pub fn insert_shuffled(&self, track: impl Into<PathBuf>) {
        self.insert(track, InsertPosition::InsertShuffled);
    }

    /// Insert a track at a random point in the tail region
    /// ([`InsertPosition::InsertLastShuffled`]).
    pub fn insert_last_shuffled(&self, track: impl Into<PathBuf>) {
        self.insert(track, InsertPosition::InsertLastShuffled);
    }

    /// Insert several tracks immediately after the current one, in order
    /// ([`InsertPosition::InsertNext`]).
    pub fn insert_tracks_next<I, P>(&self, tracks: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.insert_tracks(tracks, InsertPosition::InsertNext);
    }

    /// Append several tracks to the end of the queue, in order
    /// ([`InsertPosition::InsertLast`]).
    pub fn insert_tracks_last<I, P>(&self, tracks: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.insert_tracks(tracks, InsertPosition::InsertLast);
    }

    /// Insert several tracks at random points between the current track and
    /// the end ([`InsertPosition::InsertShuffled`]).
    pub fn insert_tracks_shuffled<I, P>(&self, tracks: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.insert_tracks(tracks, InsertPosition::InsertShuffled);
    }

    /// Insert several tracks shuffled among themselves at the tail of the
    /// queue ([`InsertPosition::InsertLastShuffled`]).
    pub fn insert_tracks_last_shuffled<I, P>(&self, tracks: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.insert_tracks(tracks, InsertPosition::InsertLastShuffled);
    }

    /// Remove the track at `index` (0-based) from the queue. An out-of-range
    /// index is ignored. Removing a track *before* the current one keeps the
    /// current track playing (the cursor shifts with it); removing the
    /// currently-playing track hard-cuts to the track that slides into its
    /// place; removing the last remaining track stops playback.
    pub fn remove(&self, index: usize) {
        let _ = self.tx.send(Command::Remove(index));
    }

    /// Empty the queue and stop playback. Also clears any saved resume state
    /// so the next launch starts fresh.
    pub fn clear_queue(&self) {
        self.send_invalidating(Command::Clear);
    }

    // ---- resume (auto-persist / restore) --------------------------------

    /// A snapshot of the current queue, in order.
    pub fn queue(&self) -> Vec<PathBuf> {
        self.shared.queue.lock().unwrap().clone()
    }

    /// Restore the queue and the exact position saved by a previous session
    /// (from `PlayerConfig::resume_file`). Returns the restored state, or
    /// `None` if resume is disabled or there is nothing to resume.
    ///
    /// Playback is **not** started — call [`Player::play`] to resume from the
    /// stored position, mirroring Rockbox's resume-on-startup.
    pub fn resume(&self) -> Option<ResumeState> {
        let path = self.resume_file.as_ref()?;
        let state = resume::load(path)?;
        self.send_invalidating(Command::Resume(state.clone()));
        Some(state)
    }

    /// Force an immediate write of the resume file (the engine also saves on
    /// pause / stop / track change / shutdown and periodically while
    /// playing). No-op when resume is disabled.
    pub fn save_resume(&self) {
        let _ = self.tx.send(Command::SaveResume);
    }

    /// Delete the resume file so the next launch starts fresh.
    pub fn clear_resume(&self) {
        if let Some(p) = &self.resume_file {
            resume::clear(p);
        }
    }

    // ---- m3u / m3u8 playlist files --------------------------------------

    /// Import an `.m3u` / `.m3u8` file into the queue at `position`
    /// (relative paths resolve against the file's directory). Returns the
    /// tracks that were read.
    pub fn import_m3u(
        &self,
        path: impl AsRef<std::path::Path>,
        position: InsertPosition,
    ) -> std::io::Result<Vec<PathBuf>> {
        let tracks = m3u::read_paths(path.as_ref())?;
        if !tracks.is_empty() || position == InsertPosition::Replace {
            let _ = self.tx.send(Command::Insert(tracks.clone(), position));
        }
        Ok(tracks)
    }

    /// Replace the queue with the contents of an `.m3u` / `.m3u8` file.
    /// Does not change playback state; call [`Player::play`] to start.
    pub fn load_m3u(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<Vec<PathBuf>> {
        let tracks = m3u::read_paths(path.as_ref())?;
        self.set_queue(tracks.clone());
        Ok(tracks)
    }

    /// Export the current queue to an `.m3u8` file (UTF-8, `#EXTM3U`
    /// header, one path per line), written atomically. Use the same path to
    /// **update** an existing playlist.
    pub fn export_m3u(&self, path: impl AsRef<std::path::Path>) -> std::io::Result<()> {
        m3u::write_paths(path.as_ref(), &self.queue())
    }

    pub fn play(&self) {
        let _ = self.tx.send(Command::Play);
    }
    pub fn pause(&self) {
        self.send_invalidating(Command::Pause);
    }
    /// Toggle play/pause.
    pub fn toggle(&self) {
        self.send_invalidating(Command::Toggle);
    }
    pub fn stop(&self) {
        self.send_invalidating(Command::Stop);
    }
    /// Skip to the next track (honours the crossfade manual-skip mode).
    pub fn next(&self) {
        self.send_invalidating(Command::Next);
    }
    /// Skip to the previous track.
    pub fn previous(&self) {
        self.send_invalidating(Command::Previous);
    }
    /// Jump to a specific queue index.
    pub fn skip_to(&self, index: usize) {
        self.send_invalidating(Command::SkipTo(index));
    }
    /// Seek within the current track.
    pub fn seek(&self, pos: Duration) {
        let _ = self.tx.send(Command::Seek(pos));
    }
    /// Set output volume, 0.0..=1.0.
    pub fn set_volume(&self, vol: f32) {
        let _ = self.tx.send(Command::SetVolume(vol));
    }
    /// Set stereo balance, -100 (full left) ..= +100 (full right); 0 = centre.
    pub fn set_balance(&self, balance: i32) {
        let _ = self.tx.send(Command::SetBalance(balance));
    }
    pub fn set_crossfade(&self, settings: CrossfadeSettings) {
        let _ = self.tx.send(Command::SetCrossfade(settings));
    }
    /// Configure ReplayGain: mode, preamp in dB, and whether to scale
    /// down to prevent clipping (using the track/album peak tag).
    pub fn set_replaygain(&self, mode: ReplayGainMode, preamp_db: f32, prevent_clipping: bool) {
        let _ = self.tx.send(Command::SetReplayGain(ReplayGainConfig {
            mode,
            preamp_db,
            prevent_clipping,
        }));
    }

    /// Enable or disable the 10-band parametric equalizer. Configure the
    /// bands with [`Player::set_eq_band`] first.
    pub fn set_eq_enabled(&self, enabled: bool) {
        let _ = self.tx.send(Command::SetEqEnabled(enabled));
    }
    /// Configure a single EQ band (`band` in `0..`[`EQ_BANDS`]; out-of-range
    /// values are ignored). Takes effect immediately; the stage must also be
    /// enabled via [`Player::set_eq_enabled`].
    pub fn set_eq_band(&self, band: usize, band_setting: EqBand) {
        let _ = self.tx.send(Command::SetEqBand(band, band_setting));
    }
    /// EQ pre-gain (headroom) in dB, applied ahead of the bands to avoid
    /// clipping.
    pub fn set_eq_precut(&self, db: f32) {
        let _ = self.tx.send(Command::SetEqPrecut(db));
    }
    /// Apply a full equalizer configuration — enable state, precut, and all
    /// bands — in a single message.
    pub fn set_equalizer(&self, eq: Equalizer) {
        let _ = self.tx.send(Command::SetEqualizer(eq));
    }
    /// Apply a built-in equalizer preset (see [`EqPreset`]) — enables the EQ
    /// and configures all ten bands in one call.
    pub fn set_eq_preset(&self, preset: EqPreset) {
        self.set_equalizer(preset.equalizer());
    }
    /// Bass/treble shelving tone controls (sets both axes and the shelf
    /// cutoffs at once).
    pub fn set_tone(&self, tone: ToneControls) {
        let _ = self.tx.send(Command::SetTone(tone));
    }
    /// Set the bass shelf gain in dB, leaving treble and the cutoffs
    /// unchanged. 0 dB is flat.
    pub fn set_bass(&self, bass_db: i32) {
        let _ = self.tx.send(Command::SetBass(bass_db));
    }
    /// Set the treble shelf gain in dB, leaving bass and the cutoffs
    /// unchanged. 0 dB is flat.
    pub fn set_treble(&self, treble_db: i32) {
        let _ = self.tx.send(Command::SetTreble(treble_db));
    }
    /// Override the bass shelf cutoff in Hz (0 = the Rockbox default of
    /// 200 Hz), leaving the gains and the treble cutoff unchanged.
    pub fn set_bass_cutoff(&self, hz: i32) {
        let _ = self.tx.send(Command::SetBassCutoff(hz));
    }
    /// Override the treble shelf cutoff in Hz (0 = the Rockbox default of
    /// 3.5 kHz), leaving the gains and the bass cutoff unchanged.
    pub fn set_treble_cutoff(&self, hz: i32) {
        let _ = self.tx.send(Command::SetTrebleCutoff(hz));
    }
    /// Headphone crossfeed (see [`Crossfeed`] / [`CrossfeedMode`]).
    pub fn set_crossfeed(&self, crossfeed: Crossfeed) {
        let _ = self.tx.send(Command::SetCrossfeed(crossfeed));
    }
    /// Haas-effect surround widening.
    pub fn set_surround(&self, surround: Surround) {
        let _ = self.tx.send(Command::SetSurround(surround));
    }
    /// Channel-mixing mode (stereo / mono / karaoke / swap / …).
    pub fn set_channel_mode(&self, mode: ChannelMode) {
        let _ = self.tx.send(Command::SetChannelMode(mode));
    }
    /// Custom stereo width in percent (100 = unchanged, 0 = mono, >100 =
    /// wider); only audible with [`ChannelMode::Custom`].
    pub fn set_stereo_width(&self, percent: i32) {
        let _ = self.tx.send(Command::SetStereoWidth(percent));
    }
    /// Perceptual Bass Enhancement (see [`BassEnhancement`]; a `strength` of
    /// 0 disables it).
    pub fn set_bass_enhancement(&self, bass_enhancement: BassEnhancement) {
        let _ = self.tx.send(Command::SetBassEnhancement(bass_enhancement));
    }
    /// Auditory Fatigue Reduction level: 0 off, 1 weak, 2 moderate, 3 strong.
    pub fn set_fatigue_reduction(&self, strength: i32) {
        let _ = self.tx.send(Command::SetFatigueReduction(strength));
    }
    /// Dynamic-range compressor (a `threshold_db` of 0 disables it).
    pub fn set_compressor(&self, compressor: Compressor) {
        let _ = self.tx.send(Command::SetCompressor(compressor));
    }
    /// Enable output dithering + noise shaping.
    pub fn set_dither(&self, enabled: bool) {
        let _ = self.tx.send(Command::SetDither(enabled));
    }
    /// Pitch/speed ratio ([`PITCH_NORMAL`] = normal); pitch and tempo shift
    /// together.
    pub fn set_pitch(&self, ratio: i32) {
        let _ = self.tx.send(Command::SetPitch(ratio));
    }

    /// Enable or disable shuffle playback. When enabled the current track keeps
    /// playing and the remaining queue is played in a shuffled order; disabling
    /// restores natural queue order from the current track.
    pub fn set_shuffle(&self, enabled: bool) {
        let _ = self.tx.send(Command::SetShuffle(enabled));
    }
    /// Set the repeat mode ([`RepeatMode::Off`] / [`RepeatMode::One`] /
    /// [`RepeatMode::All`]).
    pub fn set_repeat(&self, mode: RepeatMode) {
        let _ = self.tx.send(Command::SetRepeat(mode));
    }
    /// Whether shuffle playback is currently enabled. Because setters are
    /// asynchronous, a value set moments ago may not be reflected until the
    /// engine thread processes it.
    pub fn shuffle(&self) -> bool {
        self.shared.shuffle.load(Ordering::Relaxed)
    }
    /// The current repeat mode (see the note on [`Player::shuffle`]).
    pub fn repeat(&self) -> RepeatMode {
        RepeatMode::from_u8(self.shared.repeat.load(Ordering::Relaxed))
    }

    /// Current volume, 0.0..=1.0.
    pub fn volume(&self) -> f32 {
        self.shared.volume_f32()
    }

    /// Current stereo balance, -100 (full left) ..= +100 (full right).
    pub fn balance(&self) -> i32 {
        self.shared.balance_value()
    }

    /// A snapshot of the full DSP-chain configuration (EQ, tone, surround,
    /// channel mixing, stereo width, compressor, dither and pitch) as last
    /// applied. Because setters are asynchronous, a value set moments ago may
    /// not be reflected until the engine thread processes it.
    pub fn dsp_settings(&self) -> DspSettings {
        self.shared.dsp.lock().unwrap().clone()
    }

    /// Whether the 10-band parametric equalizer is currently enabled. A
    /// convenience shortcut for `self.dsp_settings().equalizer.enabled`.
    pub fn is_eq_enabled(&self) -> bool {
        self.shared.dsp.lock().unwrap().equalizer.enabled
    }

    /// A snapshot of the player's status.
    pub fn status(&self) -> Status {
        let state = match self.shared.state.load(Ordering::Relaxed) {
            ST_PLAYING => PlaybackState::Playing,
            ST_PAUSED => PlaybackState::Paused,
            _ => PlaybackState::Stopped,
        };
        let idx = self.shared.index.load(Ordering::Relaxed);
        let rate = self.shared.output_rate.load(Ordering::Relaxed).max(1);
        let decode_ms = self.shared.decode_pos_ms.load(Ordering::Relaxed);
        // Playback lags decoding by the ring buffer's fill.
        let ring_lag_ms = (self.shared.ring_frames() as u64 * 1000) / rate as u64;
        let position = Duration::from_millis(decode_ms.saturating_sub(ring_lag_ms));
        Status {
            state,
            index: (idx != usize::MAX).then_some(idx),
            position,
            duration: Duration::from_millis(self.shared.duration_ms.load(Ordering::Relaxed)),
            metadata: self.shared.meta.lock().unwrap().clone(),
            queue_len: self.shared.queue_len.load(Ordering::Relaxed),
            shuffle: self.shared.shuffle.load(Ordering::Relaxed),
            repeat: RepeatMode::from_u8(self.shared.repeat.load(Ordering::Relaxed)),
            levels: self.shared.levels(),
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.send_invalidating(Command::Shutdown);
        if let Some(e) = self.engine.take() {
            let _ = e.join();
        }
    }
}

/// Allocate the [`Shared`] state block from a config and the resolved output
/// rate. Backend-agnostic — used by every [`OutputConfig`] path.
fn make_shared(config: &PlayerConfig, rate: u32) -> Arc<Shared> {
    Arc::new(Shared {
        state: AtomicU8::new(ST_STOPPED),
        decode_pos_ms: AtomicU64::new(0),
        duration_ms: AtomicU64::new(0),
        index: AtomicUsize::new(usize::MAX),
        queue_len: AtomicUsize::new(0),
        target_amp: AtomicU32::new(0f32.to_bits()),
        volume: AtomicU32::new(config.volume.clamp(0.0, 1.0).to_bits()),
        balance: AtomicI32::new(0),
        balance_gain_l: AtomicU32::new(1f32.to_bits()),
        balance_gain_r: AtomicU32::new(1f32.to_bits()),
        output_rate: AtomicU32::new(rate),
        shuffle: AtomicBool::new(config.shuffle),
        repeat: AtomicU8::new(config.repeat.to_u8()),
        ring: Mutex::new(VecDeque::new()),
        level_l: AtomicU32::new(0),
        level_r: AtomicU32::new(0),
        level_bands: std::array::from_fn(|_| AtomicU32::new(0)),
        level_low_l: AtomicU32::new(0),
        level_low_r: AtomicU32::new(0),
        meta: Mutex::new(None),
        dsp: Mutex::new(config.dsp.clone()),
        queue: Mutex::new(Vec::new()),
        managed: Mutex::new(None),
        next_generation: AtomicU64::new(1),
        output_generation: AtomicU64::new(0),
        output_session: Mutex::new(None),
        output_boundary: match config.output {
            OutputConfig::Cpal => StreamOutputBoundary::DeviceBuffer,
            OutputConfig::Stdout => StreamOutputBoundary::Unavailable,
            _ => StreamOutputBoundary::ByteStream,
        },
        output_epoch: AtomicU64::new(0),
        output_nonblocking: matches!(
            config.output,
            OutputConfig::Fifo(_) | OutputConfig::Unix { .. } | OutputConfig::Tcp { .. }
        ),
        output_backpressure: AtomicU64::new(0),
        output_bytes: AtomicU64::new(0),
        output_nonzero_bytes: AtomicU64::new(0),
        output_failed: AtomicBool::new(false),
        writer_exited: AtomicBool::new(false),
    })
}

/// Spawn the engine thread and wrap everything up into a [`Player`]. Shared
/// by every backend; `output` is the already-started output handle.
fn assemble(config: PlayerConfig, rate: u32, shared: Arc<Shared>, output: OutputHandle) -> Player {
    let (tx, rx) = std::sync::mpsc::channel();
    let engine_shared = Arc::clone(&shared);
    let resume_file = config.resume_file.clone();
    let engine_cfg = EngineConfig {
        output_rate: rate,
        buffer_frames: (config.buffer_seconds.max(0.5) * rate as f32) as usize,
        crossfade: config.crossfade,
        replaygain: ReplayGainConfig {
            mode: config.replaygain_mode,
            preamp_db: config.replaygain_preamp_db,
            prevent_clipping: config.replaygain_prevent_clipping,
        },
        dsp: config.dsp,
        shuffle: config.shuffle,
        repeat: config.repeat,
        resume_file: resume_file.clone(),
        resume_save_interval: config.resume_save_interval,
    };
    let engine = std::thread::Builder::new()
        .name("rbplayback".into())
        .spawn(move || Engine::new(engine_shared, rx, engine_cfg).run())
        .expect("spawn engine thread");

    Player {
        tx,
        shared,
        _output: output,
        engine: Some(engine),
        resume_file,
    }
}

/// Drain the ring into a raw **S16LE** stereo byte stream (stdout / FIFO /
/// Unix / TCP), paced to real time with a monotonic clock so a consumer
/// that does *not* clock the stream itself (a FIFO, a socket, `ffplay -`)
/// still plays at the correct speed.
///
/// Mirrors the `cpal` callback's fade/balance semantics: while paused
/// (`target_amp == 0`) it emits **silence without draining the ring**, so
/// the buffered audio is frozen and resume is click-free — and the byte
/// stream keeps flowing so a permanent reader (Snapcast, `ffplay`) never
/// sees a gap or EOF.
fn spawn_stream_writer(
    mut writer: Box<dyn Write + Send>,
    rate: u32,
    shared: Arc<Shared>,
) -> StreamSink {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_thread = Arc::clone(&stop);
    // ~20 ms chunks: small enough for responsive transport, large enough to
    // keep syscall overhead negligible.
    let chunk_frames = (rate / 50).max(1) as usize;
    // ~1/3 second to fade the full 0..1 range, matching pcmbuf_fade_tick.
    let step = 3.0 / rate as f32;
    let frame_dur = Duration::from_secs_f64(chunk_frames as f64 / rate as f64);

    let thread = std::thread::Builder::new()
        .name("rbplayback-out".into())
        .spawn(move || {
            // 2 channels * 2 bytes/sample.
            let mut buf = vec![0u8; chunk_frames * 4];
            let mut cur_amp = 0.0f32;
            let mut meter = LevelMeter::default();
            let alpha = LevelMeter::alpha(rate);
            let band_alphas = LevelMeter::band_alphas(rate);
            let mut next = Instant::now() + frame_dur;
            let mut last_epoch = 0;
            let mut wire_offset = 0usize;

            while !stop_thread.load(Ordering::Relaxed) {
                let mut ring = shared.ring.lock().unwrap();
                let epoch = shared.output_epoch.load(Ordering::Relaxed);
                if epoch != last_epoch {
                    cur_amp = 0.0;
                    last_epoch = epoch;
                }
                let target = f32::from_bits(shared.target_amp.load(Ordering::Relaxed));
                let gain_l = f32::from_bits(shared.balance_gain_l.load(Ordering::Relaxed));
                let gain_r = f32::from_bits(shared.balance_gain_r.load(Ordering::Relaxed));

                let mut media_frames = 0;
                if target == 0.0 && cur_amp == 0.0 {
                    // Paused/stopped: emit silence, freeze the ring.
                    buf.iter_mut().for_each(|b| *b = 0);
                } else {
                    for frame in buf.chunks_mut(4) {
                        if cur_amp < target {
                            cur_amp = (cur_amp + step).min(target);
                        } else if cur_amp > target {
                            cur_amp = (cur_amp - step).max(target);
                        }
                        // Fade-out just completed: stop draining, freeze.
                        if cur_amp == 0.0 && target == 0.0 {
                            frame.fill(0);
                            continue;
                        }
                        if ring.len() >= 2 {
                            media_frames += 1;
                        }
                        let l = ring.pop_front().unwrap_or(0);
                        let r = ring.pop_front().unwrap_or(0);
                        let lv = ((l as f32) * cur_amp * gain_l).clamp(-32768.0, 32767.0) as i16;
                        let rv = ((r as f32) * cur_amp * gain_r).clamp(-32768.0, 32767.0) as i16;
                        frame[0..2].copy_from_slice(&lv.to_le_bytes());
                        frame[2..4].copy_from_slice(&rv.to_le_bytes());
                        // Measured after gain, balance and fade: what the
                        // meter shows is what leaves the output.
                        meter.push(
                            alpha,
                            &band_alphas,
                            (lv as f32 / 32768.0).abs(),
                            (rv as f32 / 32768.0).abs(),
                            (lv as f32 + rv as f32) / (2.0 * 32768.0),
                        );
                    }
                    shared.set_levels(meter.take());
                }
                drop(ring);

                if shared.output_nonblocking {
                    match write_output_chunk(
                        &mut *writer,
                        &buf,
                        media_frames,
                        epoch,
                        &shared,
                        &stop_thread,
                        &mut wire_offset,
                    ) {
                        Ok(true) => {}
                        Ok(false) => continue, // revoked: discard unsent bytes
                        Err(_) => {
                            shared.output_failed.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                } else {
                    // Legacy stdout may block. Never hold the generation gate
                    // around it or advertise cancellable output for this path.
                    if writer.write_all(&buf).is_err() || writer.flush().is_err() {
                        shared.output_failed.store(true, Ordering::Relaxed);
                        break;
                    }
                }

                // Pace to real time. If we fell behind (scheduling hiccup),
                // resync the deadline instead of trying to "catch up" in a
                // burst.
                let now = Instant::now();
                if next > now {
                    std::thread::sleep(next - now);
                }
                next += frame_dur;
                if next < now {
                    next = now + frame_dur;
                }
            }
            shared.writer_exited.store(true, Ordering::Relaxed);
        })
        .expect("spawn output writer thread");

    StreamSink {
        stop,
        thread: Some(thread),
    }
}

// Every final write is nonblocking and serialized with generation revocation.
// Bytes already accepted by the kernel cannot be recalled. Preserve S16LE
// stereo framing if cancellation lands in the middle of a partial frame.
fn write_output_chunk(
    writer: &mut dyn Write,
    pcm: &[u8],
    media_frames: usize,
    epoch: u64,
    shared: &Shared,
    stop: &AtomicBool,
    wire_offset: &mut usize,
) -> std::io::Result<bool> {
    debug_assert!(media_frames <= pcm.len() / 4);
    let padding = (4 - *wire_offset) % 4;
    let mut pending = vec![0; padding];
    pending.extend_from_slice(pcm);
    let mut offset = 0;
    while offset < pending.len() {
        if stop.load(Ordering::Relaxed) {
            return Ok(false);
        }
        let result = {
            let _gate = shared.ring.lock().unwrap();
            if shared.output_epoch.load(Ordering::Relaxed) != epoch {
                return Ok(false);
            }
            let result = writer.write(&pending[offset..]);
            if let Ok(count) = result {
                // Count only fully accepted media frames. A partial old frame,
                // alignment padding and synthesized underrun silence add no time.
                let before = (offset.saturating_sub(padding) / 4).min(media_frames);
                let after = ((offset + count).saturating_sub(padding) / 4).min(media_frames);
                shared.record_stream_output(after - before);
                shared
                    .output_bytes
                    .fetch_add(count as u64, Ordering::Relaxed);
                shared.output_nonzero_bytes.fetch_add(
                    pending[offset..offset + count]
                        .iter()
                        .filter(|b| **b != 0)
                        .count() as u64,
                    Ordering::Relaxed,
                );
            }
            result
        };
        match result {
            Ok(0) => return Err(std::io::ErrorKind::WriteZero.into()),
            Ok(count) => {
                offset += count;
                *wire_offset = (*wire_offset + count) % 4;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                shared.output_backpressure.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(true)
}

/// Build the cpal output stream: drains the ring buffer, converts i16 →
/// f32, and applies a per-sample amplitude ramp toward `target_amp`
/// (~⅓ s full-range, matching Rockbox's pause/stop fade) for click-free
/// transitions.
#[cfg(feature = "cpal")]
/// Accumulates one callback buffer's worth of level, full-band and low.
///
/// A one-pole low-pass per channel, carried across callbacks so the filter
/// does not restart every buffer, plus running sums of squares. Deliberately
/// trivial arithmetic: this runs inside the audio callback, where anything
/// that can block or allocate is a dropout.
#[derive(Default)]
struct LevelMeter {
    /// Low-pass state, one per channel.
    lp_l: f32,
    lp_r: f32,
    sum_l: f32,
    sum_r: f32,
    sum_low_l: f32,
    sum_low_r: f32,
    /// The spectrum ladder: one low-pass per cutoff over the mono mix. The
    /// difference of two neighbouring stages is a band-pass, so `ladder[k] -
    /// ladder[k+1]` is band k's signal without any real filter design.
    ladder: [f32; SPECTRUM_BANDS],
    sum_bands: [f32; SPECTRUM_BANDS],
    frames: u32,
}

impl LevelMeter {
    /// `alpha` for a one-pole low-pass at `cutoff` Hz, for a given rate.
    fn alpha_at(rate: u32, cutoff: f32) -> f32 {
        let rc = 1.0 / (std::f32::consts::TAU * cutoff);
        let dt = 1.0 / rate.max(1) as f32;
        dt / (rc + dt)
    }

    /// `alpha` for the meters' 200 Hz bass band.
    fn alpha(rate: u32) -> f32 {
        Self::alpha_at(rate, 200.0)
    }

    /// One alpha per ladder stage, precomputed outside the callback.
    fn band_alphas(rate: u32) -> [f32; SPECTRUM_BANDS] {
        std::array::from_fn(|i| Self::alpha_at(rate, BAND_CUTOFFS[i]))
    }

    /// `left` / `right` are the rectified per-channel magnitudes the meters
    /// have always eaten; `signed_mono` is the un-rectified mix, which is what
    /// the band ladder must filter — a rectified signal has its spectrum
    /// folded, and every band would just see the envelope.
    fn push(
        &mut self,
        alpha: f32,
        band_alphas: &[f32; SPECTRUM_BANDS],
        left: f32,
        right: f32,
        signed_mono: f32,
    ) {
        self.lp_l += alpha * (left - self.lp_l);
        self.lp_r += alpha * (right - self.lp_r);
        self.sum_l += left * left;
        self.sum_r += right * right;
        self.sum_low_l += self.lp_l * self.lp_l;
        self.sum_low_r += self.lp_r * self.lp_r;

        let mono = signed_mono;
        for k in 0..SPECTRUM_BANDS {
            self.ladder[k] += band_alphas[k] * (mono - self.ladder[k]);
        }
        for k in 0..SPECTRUM_BANDS {
            let band = if k + 1 < SPECTRUM_BANDS {
                self.ladder[k + 1] - self.ladder[k]
            } else {
                mono - self.ladder[k]
            };
            self.sum_bands[k] += band * band;
        }
        self.frames += 1;
    }

    /// The buffer's RMS, resetting the sums but keeping the filter state.
    fn take(&mut self) -> Levels {
        let frames = self.frames.max(1) as f32;
        let rms = |sum: f32| (sum / frames).sqrt().clamp(0.0, 1.0);
        let levels = Levels {
            left: rms(self.sum_l),
            right: rms(self.sum_r),
            // Low-passing removes most of the energy, so the band is scaled
            // to reach the top of a meter on bass-heavy material rather than
            // hovering near the floor.
            low_left: (rms(self.sum_low_l) * 3.0).clamp(0.0, 1.0),
            low_right: (rms(self.sum_low_r) * 3.0).clamp(0.0, 1.0),
            // Narrow bands carry little of the total energy; the same
            // reach-the-top scaling as the bass band, a little stronger.
            bands: std::array::from_fn(|k| (rms(self.sum_bands[k]) * 4.0).clamp(0.0, 1.0)),
        };
        self.sum_l = 0.0;
        self.sum_r = 0.0;
        self.sum_low_l = 0.0;
        self.sum_low_r = 0.0;
        self.sum_bands = [0.0; SPECTRUM_BANDS];
        self.frames = 0;
        levels
    }
}

fn build_stream(
    device: &cpal::Device,
    rate: u32,
    shared: Arc<Shared>,
) -> Result<cpal::Stream, Error> {
    let config = cpal::StreamConfig {
        channels: 2,
        sample_rate: cpal::SampleRate(rate),
        buffer_size: cpal::BufferSize::Default,
    };
    // ~1/3 second to fade the full 0..1 range, matching pcmbuf_fade_tick.
    let step = 3.0 / rate as f32;
    let mut cur_amp = 0.0f32;
    let mut meter = LevelMeter::default();
    let alpha = LevelMeter::alpha(rate);
    let band_alphas = LevelMeter::band_alphas(rate);

    let mut last_epoch = 0;
    let err_fn = |e| eprintln!("rockbox-playback: output stream error: {e}");
    let stream = device
        .build_output_stream(
            &config,
            move |data: &mut [f32], _| {
                let mut ring = shared.ring.lock().unwrap();
                let epoch = shared.output_epoch.load(Ordering::Relaxed);
                if epoch != last_epoch {
                    cur_amp = 0.0;
                    last_epoch = epoch;
                }
                let target = f32::from_bits(shared.target_amp.load(Ordering::Relaxed));
                let gain_l = f32::from_bits(shared.balance_gain_l.load(Ordering::Relaxed));
                let gain_r = f32::from_bits(shared.balance_gain_r.load(Ordering::Relaxed));
                // Fully muted for pause/stop: output silence WITHOUT
                // consuming the ring, so the buffered audio is frozen and
                // resume is click-free from where it left off.
                if target == 0.0 && cur_amp == 0.0 {
                    data.fill(0.0);
                    // Silence is a level too: a meter left holding its last
                    // reading looks stuck rather than stopped.
                    shared.set_levels(Levels::default());
                    return;
                }
                let mut media_frames = 0;
                for frame in data.chunks_mut(2) {
                    if cur_amp < target {
                        cur_amp = (cur_amp + step).min(target);
                    } else if cur_amp > target {
                        cur_amp = (cur_amp - step).max(target);
                    }
                    // Reached full mute mid-callback (fade-out complete):
                    // stop draining and freeze the rest.
                    if cur_amp == 0.0 && target == 0.0 {
                        frame.fill(0.0);
                        continue;
                    }
                    if ring.len() >= 2 && frame.len() == 2 {
                        media_frames += 1;
                    }
                    let l = ring.pop_front().unwrap_or(0);
                    let r = ring.pop_front().unwrap_or(0);
                    frame[0] = (l as f32 / 32768.0) * cur_amp * gain_l;
                    if frame.len() > 1 {
                        frame[1] = (r as f32 / 32768.0) * cur_amp * gain_r;
                    }
                    // Measured here, after gain, balance and fade: what the
                    // meter shows is what leaves the device.
                    meter.push(
                        alpha,
                        &band_alphas,
                        frame[0].abs(),
                        frame.get(1).copied().unwrap_or(0.0).abs(),
                        (frame[0] + frame.get(1).copied().unwrap_or(frame[0])) * 0.5,
                    );
                }
                shared.record_stream_output(media_frames);
                shared.set_levels(meter.take());
            },
            err_fn,
            None,
        )
        .map_err(|e| Error::Stream(e.to_string()))?;
    Ok(stream)
}

struct EngineConfig {
    output_rate: u32,
    buffer_frames: usize,
    crossfade: CrossfadeSettings,
    replaygain: ReplayGainConfig,
    dsp: DspSettings,
    shuffle: bool,
    repeat: RepeatMode,
    resume_file: Option<PathBuf>,
    resume_save_interval: Duration,
}

struct Engine {
    shared: Arc<Shared>,
    rx: Receiver<Command>,
    cfg: EngineConfig,
    dsp: rockbox_dsp::Dsp,
    queue: Vec<PathBuf>,
    index: usize,
    /// Whether shuffle playback is enabled.
    shuffle: bool,
    /// The current repeat mode.
    repeat: RepeatMode,
    /// Play order: a permutation of `0..queue.len()` giving the sequence in
    /// which queue indices are played. Identity when shuffle is off; when on,
    /// the current track stays first and the rest are shuffled. Rebuilt on
    /// every queue change and shuffle toggle (see [`Engine::rebuild_play_order`]).
    order: Vec<usize>,
    /// Open decoder for the current track, if playing.
    decoder: Option<Decoder>,
    playing: bool,
    paused: bool,
    /// The queue is fully decoded; the engine is draining the ring buffer
    /// before going Stopped.
    finishing: bool,
    /// Native rate of the current decoder's output (0 = unknown); the DSP
    /// resampler is only reconfigured when this changes.
    input_rate: u32,
    /// Microseconds of audio decoded so far in the current track, counted
    /// from the chunks themselves. Fallback position source when the codec
    /// doesn't report elapsed time (e.g. `aac_bsf` on a bitrate-less
    /// HLS/DASH stream leaves it to the host).
    decoded_us: u64,
    /// Live copy of the DSP-chain settings, mirrored into [`Shared::dsp`]
    /// after every change. Also lets `set_bass`/`set_treble` change one tone
    /// axis while preserving the other (the DSP stage recomputes its prescale
    /// from both values at once).
    dsp_state: DspSettings,
    /// Set for the duration of a manual-skip crossfade so [`Engine::next_index`]
    /// resolves to the skip target instead of `index + 1`.
    pending_manual_target: Option<usize>,
    /// Position of the last inserted track, so successive
    /// [`InsertPosition::Insert`]s append to the same block (mirrors
    /// Rockbox's `last_insert_pos`). `None` once invalidated.
    last_insert_pos: Option<usize>,
    /// xorshift64 state for the shuffled insertion positions.
    rng_state: u64,
    /// Set once a `Shutdown` command is seen so every loop — including the
    /// inner `push_frames` back-off, which may consume the command before the
    /// main loop does — unwinds instead of decoding forever.
    shutdown: bool,
    /// A `(index, position)` seek to apply the next time `index`'s decoder is
    /// opened — set by a resume restore so the track starts at the exact
    /// saved position. Discarded if the user navigates elsewhere first.
    pending_seek: Option<(usize, Duration)>,
    /// Last time the resume file was written (for interval throttling).
    last_save: Instant,
    /// Keeps the current track's HTTP cache (a temp file) alive for as long as
    /// its decoder is open. Dropped — and the temp file deleted — when the
    /// track is reset. Boxed as `Any` so the field needn't be `cfg`-gated.
    current_source: Option<Box<dyn std::any::Any + Send>>,
    managed: Option<StreamSession>,
    /// Live-radio (ICY) metadata for the current stream, if any: the station
    /// base metadata plus a handle to the changing `StreamTitle`.
    #[cfg(feature = "http")]
    current_icy: Option<IcyLive>,
}

/// Tracks a live stream's ICY metadata so the engine can refresh
/// [`Shared::meta`] as the `StreamTitle` changes.
#[cfg(feature = "http")]
struct IcyLive {
    /// Shared, live-updated current `StreamTitle`.
    title: std::sync::Arc<Mutex<Option<String>>>,
    /// Station-derived base metadata (codec, station name, genre, bitrate).
    base: Metadata,
    /// Last title reflected into `Shared::meta`, to detect changes.
    last_title: Option<String>,
}

impl Engine {
    fn new(shared: Arc<Shared>, rx: Receiver<Command>, cfg: EngineConfig) -> Self {
        let mut dsp = rockbox_dsp::Dsp::new(cfg.output_rate);
        apply_replaygain_mode(&mut dsp, &cfg.replaygain);
        apply_dsp_settings(&mut dsp, &cfg.dsp);
        let dsp_state = cfg.dsp.clone();
        let (shuffle, repeat) = (cfg.shuffle, cfg.repeat);
        Engine {
            shared,
            rx,
            cfg,
            dsp,
            queue: Vec::new(),
            index: 0,
            shuffle,
            repeat,
            order: Vec::new(),
            decoder: None,
            playing: false,
            paused: false,
            finishing: false,
            input_rate: 0,
            decoded_us: 0,
            dsp_state,
            pending_manual_target: None,
            last_insert_pos: None,
            rng_state: seed_rng(),
            shutdown: false,
            pending_seek: None,
            last_save: Instant::now(),
            current_source: None,
            managed: None,
            #[cfg(feature = "http")]
            current_icy: None,
        }
    }

    fn run(mut self) {
        loop {
            // Break even if a Shutdown was consumed by an inner loop
            // (e.g. push_frames) rather than the pump below.
            if self.shutdown {
                break;
            }
            // Drain pending commands; returns false on Shutdown.
            if !self.pump_commands(false) {
                break;
            }

            if self
                .managed
                .as_ref()
                .is_some_and(StreamSession::is_cancelled)
            {
                self.reset_current();
                self.playing = false;
                self.set_state(ST_STOPPED);
            }
            // Retain ended managed streams and position; no auto-advance.
            if self.managed.is_some() && self.decoder.is_none() {
                if self.shared.ring_frames() == 0 {
                    self.set_state(ST_STOPPED);
                }
                match self.rx.recv_timeout(Duration::from_millis(25)) {
                    Ok(cmd) => {
                        if !self.handle(cmd) {
                            break;
                        }
                    }
                    Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(_) => {}
                }
                continue;
            }

            // Paused: keep the decoder and buffered audio; idle until a
            // command changes things.
            if self.paused {
                self.set_state(ST_PAUSED);
                match self.rx.recv() {
                    Ok(cmd) => {
                        if !self.handle(cmd) {
                            break;
                        }
                    }
                    Err(_) => break,
                }
                continue;
            }

            // Finishing: the queue is decoded; stay Playing until the ring
            // drains, then Stopped. (`position` keeps advancing because the
            // ring lag shrinks.)
            if self.finishing {
                if self.shared.ring_frames() > 0 {
                    self.set_state(ST_PLAYING);
                    std::thread::sleep(Duration::from_millis(25));
                } else {
                    self.finishing = false;
                    self.playing = false;
                    // Queue played to the end — don't resume a finished
                    // playlist next launch. Clear before signalling Stopped so
                    // an observer never sees Stopped with a stale file.
                    self.clear_resume();
                    self.set_state(ST_STOPPED);
                    self.shared.decode_pos_ms.store(0, Ordering::Relaxed);
                    self.shared.index.store(usize::MAX, Ordering::Relaxed);
                    self.shared
                        .target_amp
                        .store(0f32.to_bits(), Ordering::Relaxed);
                }
                continue;
            }

            if !self.playing || (self.queue.is_empty() && self.managed.is_none()) {
                self.set_state(ST_STOPPED);
                // Idle: block until a command arrives.
                match self.rx.recv() {
                    Ok(cmd) => {
                        if !self.handle(cmd) {
                            break;
                        }
                    }
                    Err(_) => break,
                }
                continue;
            }

            // Ensure a decoder is open for the current track.
            if self.decoder.is_none() && !self.open_current() {
                // Failed to open — skip forward, or stop at end.
                if !self.advance_index(true) {
                    self.finish_queue();
                }
                continue;
            }

            self.set_state(ST_PLAYING);
            self.decode_step();

            // Publish live-radio (ICY) StreamTitle changes to status().
            #[cfg(feature = "http")]
            self.refresh_icy();

            // Refresh the resume file periodically so a crash loses at most
            // `resume_save_interval` of position.
            if self.cfg.resume_file.is_some()
                && self.last_save.elapsed() >= self.cfg.resume_save_interval
            {
                self.save_resume();
            }
        }
    }

    /// Process one decode step: pull a chunk, run DSP, and either push it
    /// or trigger a crossfade near the track's end.
    fn decode_step(&mut self) {
        // Near-end crossfade detection (auto skip).
        if self.managed.is_none() && self.should_start_crossfade() {
            self.crossfade_to_next(true);
            return;
        }

        let chunk = match self.next_output_chunk() {
            Some(c) => c,
            None => {
                if let Some(session) = self.managed.clone() {
                    let code = self.decoder.as_ref().and_then(Decoder::status);
                    session.decoder_ended(code);
                    drop(self.decoder.take());
                    session.decoder_joined();
                    self.playing = false;
                    return;
                }
                // End of track — advance (auto). Ring keeps this track's
                // buffered tail so playback is gapless.
                self.dsp.flush();
                if !self.advance_index(true) {
                    self.finish_queue();
                }
                self.decoder = None;
                return;
            }
        };
        self.push_frames(&chunk);
    }

    /// Pull one decoded chunk and run it through the DSP (ReplayGain +
    /// resample to the output rate). Returns interleaved stereo i16 at
    /// `output_rate`, or `None` at end of track.
    fn next_output_chunk(&mut self) -> Option<Vec<i16>> {
        let dec = self.decoder.as_mut()?;
        let chunk = dec.next_chunk()?;
        // Engage the resampler only when the native rate actually changes.
        if chunk.sample_rate != self.input_rate {
            self.dsp.set_input_frequency(chunk.sample_rate);
            self.input_rate = chunk.sample_rate;
        }
        if chunk.sample_rate > 0 {
            let frames = (chunk.pcm.len() / 2) as u64;
            self.decoded_us += frames * 1_000_000 / chunk.sample_rate as u64;
        }
        // Prefer the codec's own elapsed time; some codecs can't report one
        // (aac_bsf without a known bitrate) and leave it at 0 — fall back to
        // the frame count decoded so far.
        let codec_ms = dec.elapsed().as_millis() as u64;
        let pos_ms = if codec_ms > 0 {
            codec_ms
        } else {
            self.decoded_us / 1000
        };
        self.shared.decode_pos_ms.store(pos_ms, Ordering::Relaxed);
        let mut out = Vec::new();
        self.dsp.process(&chunk.pcm, &mut out);
        Some(out)
    }

    fn should_start_crossfade(&self) -> bool {
        if !self.cfg.crossfade.mode.applies(true) {
            return false;
        }
        if self.next_index(true).is_none() {
            return false; // last track
        }
        let dur = self.shared.duration_ms.load(Ordering::Relaxed);
        if dur == 0 {
            return false; // unknown length — can't predict the tail
        }
        let region_ms = (self.cfg.crossfade.region_frames(self.cfg.output_rate) as u64 * 1000)
            / self.cfg.output_rate as u64;
        let pos = self.shared.decode_pos_ms.load(Ordering::Relaxed);
        pos + region_ms >= dur
    }

    /// Perform a crossfade into the next track: gather the outgoing tail,
    /// decode the incoming head, mix, and continue with the incoming
    /// track as current.
    fn crossfade_to_next(&mut self, auto: bool) {
        let region = self.cfg.crossfade.region_frames(self.cfg.output_rate);

        // 1. Outgoing tail (post-DSP). For auto skip we're near the end,
        //    so drain what's left; for a manual skip decode `region`
        //    frames from the current position.
        let mut tail: Vec<i16> = Vec::with_capacity(region * 2);
        while tail.len() < region * 2 {
            match self.next_output_chunk() {
                Some(c) => tail.extend_from_slice(&c),
                None => break, // track ended
            }
        }
        // If the outgoing track had more than the overlap (manual skip
        // mid-track), keep only its final `region` frames as the tail;
        // anything earlier plays normally first.
        if tail.len() > region * 2 {
            let split = tail.len() - region * 2;
            let head_part: Vec<i16> = tail.drain(..split).collect();
            self.push_frames(&head_part);
        }

        // 2. Switch to the incoming track.
        let next = match self.next_index(auto) {
            Some(i) => i,
            None => {
                self.push_frames(&tail);
                self.dsp.flush();
                self.decoder = None;
                self.finish_queue();
                return;
            }
        };
        self.dsp.flush();
        self.index = next;
        self.decoder = None;
        if !self.open_current() {
            self.push_frames(&tail);
            self.advance_index(auto);
            return;
        }

        // 3. Incoming head (post-DSP), `region` frames.
        let mut head: Vec<i16> = Vec::with_capacity(region * 2);
        while head.len() < region * 2 {
            match self.next_output_chunk() {
                Some(c) => head.extend_from_slice(&c),
                None => break,
            }
        }

        // 4. Mix the overlap and emit; the incoming track continues from
        //    where its head left off.
        let mixed = crossfade::mix(&tail, &head, &self.cfg.crossfade, self.cfg.output_rate);
        self.push_frames(&mixed);
    }

    /// Push frames to the ring, sleeping while it is full but staying
    /// responsive to commands.
    fn push_frames(&mut self, pcm: &[i16]) {
        let generation = self.managed.as_ref().map(StreamSession::generation);
        let cap = self.cfg.buffer_frames * 2;
        let mut pos = 0;
        while pos < pcm.len() {
            // Back off if the ring is full, but keep handling commands. If
            // a command changes what we're doing, abandon the rest.
            loop {
                if !self.pump_commands(true) {
                    return; // shutdown
                }
                if generation != self.managed.as_ref().map(StreamSession::generation)
                    || self
                        .managed
                        .as_ref()
                        .is_some_and(StreamSession::is_cancelled)
                    || !self.playing
                    || self.decoder.is_none()
                {
                    return; // stopped / seeked / skipped — drop stale audio
                }
                let len = self.shared.ring.lock().unwrap().len();
                if len < cap {
                    break;
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            let mut ring = self.shared.ring.lock().unwrap();
            // Recheck under the gate: cancellation can race the check above.
            if self.shared.output_generation.load(Ordering::Relaxed) != generation.unwrap_or(0) {
                return;
            }
            let space = cap.saturating_sub(ring.len());
            let end = (pos + space).min(pcm.len());
            ring.extend(&pcm[pos..end]);
            pos = end;
        }
    }

    // ---- command handling ------------------------------------------------

    /// Handle all queued commands. `nonblocking` = drain without waiting.
    /// Returns false if a Shutdown was received.
    fn pump_commands(&mut self, _nonblocking: bool) -> bool {
        while let Ok(cmd) = self.rx.try_recv() {
            if !self.handle(cmd) {
                return false;
            }
        }
        true
    }

    /// Returns false on Shutdown.
    fn handle(&mut self, cmd: Command) -> bool {
        match cmd {
            Command::OpenStream(input) => {
                self.open_managed(input);
            }
            Command::Shutdown => {
                // Persist the exact position on exit, like Rockbox saves on
                // power-off — but only mid-session (a finished queue already
                // cleared its resume file).
                if self.playing || self.paused {
                    self.save_resume();
                }
                self.reset_current();
                self.shutdown = true;
                return false;
            }
            Command::SetQueue(q) => {
                self.queue = q;
                self.index = 0;
                self.finishing = false;
                self.paused = false;
                self.last_insert_pos = None;
                self.pending_seek = None;
                self.reset_current();
                self.sync_queue();
            }
            Command::Enqueue(p) => {
                self.queue.push(p);
                self.sync_queue();
            }
            Command::Insert(tracks, position) => self.insert_tracks(tracks, position),
            Command::Remove(index) => self.remove_track(index),
            Command::Clear => self.clear_queue(),
            Command::Resume(state) => {
                self.queue = state.tracks;
                self.index = state.index.min(self.queue.len().saturating_sub(1));
                self.pending_seek = Some((self.index, state.elapsed));
                self.finishing = false;
                self.paused = false;
                self.last_insert_pos = None;
                self.reset_current();
                self.sync_queue();
                // Reflect the restored index in status before play().
                self.shared.index.store(self.index, Ordering::Relaxed);
            }
            Command::SaveResume => self.save_resume(),
            Command::Play => {
                if self.managed.is_some() {
                    return true;
                }
                if self.queue.is_empty() {
                    return true;
                }
                // Resume from pause, or (re)start if stopped/finished.
                if self.paused {
                    self.set_paused(false);
                } else {
                    if self.finishing {
                        self.finishing = false;
                        self.index = 0;
                        self.reset_current();
                    }
                    self.playing = true;
                    self.shared
                        .target_amp
                        .store(self.shared.volume_f32().to_bits(), Ordering::Relaxed);
                }
            }
            Command::Pause => self.set_paused(true),
            Command::Toggle => self.set_paused(!self.paused),
            Command::Stop => self.stop_playback(),
            Command::Next => self.manual_skip(self.next_index(false)),
            Command::Previous => {
                // Step back in play order; at the start (no wrap) restart the
                // current track, matching the common "press prev to restart".
                let prev =
                    next_in_order(&self.order, self.index, self.repeat, false).or(Some(self.index));
                self.manual_skip(prev);
            }
            Command::SkipTo(i) => {
                if i < self.queue.len() {
                    self.manual_skip(Some(i));
                }
            }
            Command::Seek(pos) => self.seek(pos),
            Command::SetVolume(v) => {
                self.shared.set_volume_bits(v);
                if self.playing && self.shared.state.load(Ordering::Relaxed) == ST_PLAYING {
                    self.shared
                        .target_amp
                        .store(self.shared.volume_f32().to_bits(), Ordering::Relaxed);
                }
            }
            Command::SetBalance(b) => self.shared.set_balance_value(b),
            Command::SetCrossfade(s) => self.cfg.crossfade = s,
            Command::SetReplayGain(rg) => {
                self.cfg.replaygain = rg;
                apply_replaygain_mode(&mut self.dsp, &rg);
                // Re-apply the current track's per-track gains under the
                // new mode.
                if let Some(meta) = self.shared.meta.lock().unwrap().as_ref() {
                    apply_replaygain_track(&mut self.dsp, meta);
                }
            }
            Command::SetEqEnabled(enabled) => {
                self.dsp_state.equalizer.enabled = enabled;
                self.dsp.eq_enable(enabled);
                self.sync_dsp();
            }
            Command::SetEqBand(band, b) => {
                if band < EQ_BANDS {
                    upsert_eq_band(&mut self.dsp_state.equalizer.bands, band, b);
                    self.dsp.set_eq_band(band, b.cutoff_hz, b.q, b.gain_db);
                    self.sync_dsp();
                }
            }
            Command::SetEqPrecut(db) => {
                self.dsp_state.equalizer.precut_db = db;
                self.dsp.set_eq_precut(db);
                self.sync_dsp();
            }
            Command::SetEqualizer(eq) => {
                self.dsp_state.equalizer = eq.clone();
                apply_equalizer(&mut self.dsp, &eq);
                self.sync_dsp();
            }
            Command::SetTone(tone) => {
                self.dsp_state.tone = tone;
                apply_tone(&mut self.dsp, tone);
                self.sync_dsp();
            }
            Command::SetBass(db) => {
                self.dsp_state.tone.bass_db = db;
                apply_tone(&mut self.dsp, self.dsp_state.tone);
                self.sync_dsp();
            }
            Command::SetTreble(db) => {
                self.dsp_state.tone.treble_db = db;
                apply_tone(&mut self.dsp, self.dsp_state.tone);
                self.sync_dsp();
            }
            Command::SetBassCutoff(hz) => {
                self.dsp_state.tone.bass_cutoff_hz = hz;
                apply_tone(&mut self.dsp, self.dsp_state.tone);
                self.sync_dsp();
            }
            Command::SetTrebleCutoff(hz) => {
                self.dsp_state.tone.treble_cutoff_hz = hz;
                apply_tone(&mut self.dsp, self.dsp_state.tone);
                self.sync_dsp();
            }
            Command::SetCrossfeed(cf) => {
                self.dsp_state.crossfeed = cf;
                apply_crossfeed(&mut self.dsp, cf);
                self.sync_dsp();
            }
            Command::SetSurround(s) => {
                self.dsp_state.surround = s;
                apply_surround(&mut self.dsp, s);
                self.sync_dsp();
            }
            Command::SetChannelMode(mode) => {
                self.dsp_state.channel_mode = mode;
                self.dsp.set_channel_config(mode.to_raw());
                self.sync_dsp();
            }
            Command::SetStereoWidth(pct) => {
                self.dsp_state.stereo_width = pct;
                self.dsp.set_stereo_width(pct);
                self.sync_dsp();
            }
            Command::SetBassEnhancement(pbe) => {
                self.dsp_state.bass_enhancement = pbe;
                apply_bass_enhancement(&mut self.dsp, pbe);
                self.sync_dsp();
            }
            Command::SetFatigueReduction(strength) => {
                self.dsp_state.fatigue_reduction = strength;
                self.dsp.afr_enable(strength);
                self.sync_dsp();
            }
            Command::SetCompressor(c) => {
                self.dsp_state.compressor = c;
                self.dsp.set_compressor(&c.to_raw());
                self.sync_dsp();
            }
            Command::SetDither(enabled) => {
                self.dsp_state.dither = enabled;
                self.dsp.dither_enable(enabled);
                self.sync_dsp();
            }
            Command::SetPitch(ratio) => {
                self.dsp_state.pitch = ratio;
                self.dsp.set_pitch(ratio);
                self.sync_dsp();
            }
            Command::SetShuffle(enabled) => {
                if enabled != self.shuffle {
                    self.shuffle = enabled;
                    self.rebuild_play_order();
                }
                self.sync_modes();
            }
            Command::SetRepeat(mode) => {
                self.repeat = mode;
                self.sync_modes();
            }
        }
        true
    }

    /// Publish the current DSP-chain state to [`Shared::dsp`] so the public
    /// [`Player::dsp_settings`] handle can read it.
    fn sync_dsp(&self) {
        *self.shared.dsp.lock().unwrap() = self.dsp_state.clone();
    }

    fn set_paused(&mut self, paused: bool) {
        if paused {
            // Pause is valid while actively playing OR while draining the
            // ring after the queue finished (audio is still audible).
            if self.playing || self.finishing {
                self.paused = true;
                self.set_state(ST_PAUSED);
                self.shared
                    .target_amp
                    .store(0f32.to_bits(), Ordering::Relaxed);
                self.save_resume();
            }
        } else if self.paused || (!self.playing && !self.queue.is_empty()) {
            self.paused = false;
            if !self.finishing {
                self.playing = true;
            }
            self.set_state(ST_PLAYING);
            self.shared
                .target_amp
                .store(self.shared.volume_f32().to_bits(), Ordering::Relaxed);
        }
    }

    fn stop_playback(&mut self) {
        // Save the position before tearing down so a manual stop can still be
        // resumed (Rockbox keeps its last resume info on stop).
        if self.playing || self.paused || self.finishing {
            self.save_resume();
        }
        self.playing = false;
        self.paused = false;
        self.finishing = false;
        self.reset_current();
        self.set_state(ST_STOPPED);
        self.shared
            .target_amp
            .store(0f32.to_bits(), Ordering::Relaxed);
        self.shared.decode_pos_ms.store(0, Ordering::Relaxed);
        self.shared.index.store(usize::MAX, Ordering::Relaxed);
    }

    /// The whole queue has been decoded; keep the target amplitude up so
    /// the buffered tail plays out, and let [`Engine::run`] drain the ring
    /// before going Stopped.
    fn finish_queue(&mut self) {
        self.decoder = None;
        self.finishing = true;
        self.playing = true;
        self.shared
            .target_amp
            .store(self.shared.volume_f32().to_bits(), Ordering::Relaxed);
    }

    /// Manual (user-initiated) skip to `target`.
    fn manual_skip(&mut self, target: Option<usize>) {
        let Some(target) = target else {
            return;
        };
        if !self.playing {
            self.index = target;
            self.reset_current();
            return;
        }
        if self.cfg.crossfade.mode.applies(false) && self.decoder.is_some() {
            // Crossfade from the current position into `target`.
            // next_index(false) is `target`; align index bookkeeping.
            self.pending_manual_target = Some(target);
            self.crossfade_to_next(false);
            self.pending_manual_target = None;
        } else {
            // Immediate: drop the current decoder and clear buffered audio.
            self.index = target;
            self.reset_current();
            self.shared.ring.lock().unwrap().clear();
        }
    }

    fn seek(&mut self, pos: Duration) {
        if self.managed.is_some() {
            return;
        }
        if let Some(dec) = self.decoder.as_mut() {
            dec.seek(pos);
            self.decoded_us = pos.as_micros() as u64;
            self.dsp.flush();
            self.shared.ring.lock().unwrap().clear();
            self.shared
                .decode_pos_ms
                .store(pos.as_millis() as u64, Ordering::Relaxed);
            self.save_resume();
        }
    }

    // ---- queue / decoder helpers ----------------------------------------

    fn open_managed(&mut self, input: managed::StreamInput) {
        self.reset_current();
        self.queue.clear();
        self.sync_queue();
        self.index = 0;
        self.pending_seek = None;
        self.finishing = false;
        self.paused = false;
        let session = input.session.clone();
        self.managed = Some(session.clone());
        if session.is_cancelled() {
            drop(input);
            session.decoder_joined();
            return;
        }
        session.set_phase(StreamPhase::Opening);
        let managed::StreamInput {
            reader,
            format_ext,
            metadata,
            ..
        } = input;
        match Decoder::open_stream(Box::new(reader), &format_ext, metadata) {
            Ok(dec) if !session.is_cancelled() => {
                self.install_decoder(dec, false);
                self.shared.index.store(usize::MAX, Ordering::Relaxed);
                self.playing = true;
                {
                    let _gate = self.shared.ring.lock().unwrap();
                    if self.shared.output_generation.load(Ordering::Relaxed) == session.generation()
                    {
                        self.shared
                            .target_amp
                            .store(self.shared.volume_f32().to_bits(), Ordering::Relaxed);
                    }
                }
                session.set_phase(StreamPhase::Decoding);
            }
            Ok(dec) => {
                drop(dec);
                session.decoder_joined();
            }
            Err(_) => {
                session.set_phase(StreamPhase::Failed);
                session.decoder_joined();
            }
        }
    }

    fn open_current(&mut self) -> bool {
        let Some(path) = self.queue.get(self.index).cloned() else {
            return false;
        };
        let s = path.to_string_lossy().into_owned();
        // Any previous stream's live metadata no longer applies.
        #[cfg(feature = "http")]
        {
            self.current_icy = None;
        }

        // A remote URL is either a seekable finite file (fetched into a cache
        // and decoded like a local file) or an unbounded live stream (decoded
        // forward-only). Local paths open directly.
        if source::is_url(&s) {
            match self.open_remote(&s) {
                Some(ok) => return ok,
                None => return false,
            }
        }
        self.current_source = None;
        let dec = match Decoder::open(&path) {
            Ok(d) => d,
            Err(_) => return false,
        };
        self.install_decoder(dec, /* seekable = */ true)
    }

    /// Open an `http(s)://` URL. Returns `Some(true/false)` once handled (the
    /// bool is decoder-open success), or `None` when the `http` feature is off.
    fn open_remote(&mut self, url: &str) -> Option<bool> {
        #[cfg(feature = "http")]
        {
            match source::open_remote(url) {
                Ok(source::Remote::File(mut src)) => {
                    self.current_source = None;
                    // Buffer only the header via range requests, then decode
                    // the rest on demand — no full download, playback starts
                    // as soon as the header is present. ~512 KiB covers the
                    // format/rate/duration for the common codecs.
                    const HEADER_BYTES: u64 = 512 * 1024;
                    const TAIL_BYTES: u64 = 1024 * 1024;
                    if src.prefetch(HEADER_BYTES).is_err() {
                        return Some(false);
                    }
                    let size = src.size();
                    // Parse tags/duration from the prefetched header (the cache
                    // file is full-size and sparse, so this reads what's there).
                    let mut meta = rockbox_metadata::read(src.cache_path()).unwrap_or_default();
                    // An MP3 can carry an ID3v2 tag (cover art) larger than the
                    // header window, leaving the first MPEG frame — and the
                    // Xing/VBR header that holds the duration — unfetched.
                    // Extend the front past the tag and retry.
                    if meta.duration.is_zero() && size > HEADER_BYTES {
                        if let Some(tag_len) = source::id3v2_len(src.cache_path()) {
                            let want = tag_len.saturating_add(256 * 1024).min(size);
                            if want > HEADER_BYTES && src.prefetch_range(0, want).is_ok() {
                                meta = rockbox_metadata::read(src.cache_path()).unwrap_or_default();
                            }
                        }
                    }
                    // A non-faststart MP4 keeps `moov` at EOF, so the header
                    // window can't reach it. Walk the atom chain to find the
                    // box and fetch exactly it — a fixed tail guess misses the
                    // ones whose `moov` is bigger than the guess. The parser
                    // seeks over `mdat` rather than reading it, so header +
                    // `moov` is enough and the hole between is never touched.
                    if meta.duration.is_zero() && size > HEADER_BYTES {
                        if let Some((start, end)) = source::mp4_moov_extent(&mut src) {
                            if src.prefetch_range(start, end).is_ok() {
                                meta = rockbox_metadata::read(src.cache_path()).unwrap_or_default();
                            }
                        }
                    }
                    // Last resort for a container whose chain we can't walk:
                    // take the tail and hope the trailer lives there.
                    if meta.duration.is_zero() && size > HEADER_BYTES {
                        let tail = size.saturating_sub(TAIL_BYTES).max(HEADER_BYTES);
                        if src.prefetch_range(tail, size).is_ok() {
                            meta = rockbox_metadata::read(src.cache_path()).unwrap_or_default();
                        }
                    }
                    let ext = src
                        .cache_path()
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_string();
                    let freq = meta.sample_rate;
                    match Decoder::open_seekable(Box::new(src), size, freq, &ext, meta) {
                        Ok(dec) => Some(self.install_decoder(dec, true)),
                        Err(_) => Some(false),
                    }
                }
                Ok(source::Remote::Adaptive(stream)) => {
                    // HLS / MPEG-DASH: segments arrive as one forward-only
                    // demuxed bitstream. VOD presentations know their
                    // duration up front; live ones play until the origin
                    // stops. No seeking either way (forward-only stream).
                    self.current_source = None;
                    let ext = stream.format_ext().to_string();
                    let mut base = Metadata::default();
                    base.codec = format!("{} {}", stream.kind_label(), ext.to_uppercase());
                    base.sample_rate = stream.sample_rate();
                    if let Some(d) = stream.duration() {
                        base.duration = d;
                    }
                    match Decoder::open_stream(Box::new(stream), &ext, base) {
                        Ok(dec) => Some(self.install_decoder(dec, false)),
                        Err(_) => Some(false),
                    }
                }
                Ok(source::Remote::Stream(stream)) => {
                    // Unbounded: no random access, so skip get_metadata and
                    // decode the response body forward-only.
                    self.current_source = None;
                    let ext = stream.format_ext().to_string();
                    // Seed metadata from the ICY station headers; the changing
                    // per-song StreamTitle is refreshed from `current_icy`.
                    let station = stream.station().clone();
                    let mut base = Metadata::default();
                    base.codec = ext.to_uppercase();
                    base.album = station.name.clone().unwrap_or_default();
                    base.genre = station.genre.clone().unwrap_or_default();
                    base.bitrate = station.bitrate.unwrap_or(0);
                    self.current_icy = Some(IcyLive {
                        title: stream.title(),
                        base: base.clone(),
                        last_title: None,
                    });
                    match Decoder::open_stream(Box::new(stream), &ext, base) {
                        Ok(dec) => Some(self.install_decoder(dec, false)),
                        Err(_) => Some(false),
                    }
                }
                Err(_) => Some(false),
            }
        }
        #[cfg(not(feature = "http"))]
        {
            let _ = url;
            None
        }
    }

    /// Wire an opened decoder into engine state. `seekable` gates the
    /// resume-position seek (a live stream can't seek). Returns `true`.
    fn install_decoder(&mut self, mut dec: Decoder, seekable: bool) -> bool {
        self.input_rate = 0; // force resampler reconfigure on first chunk
        let meta = dec.metadata().clone();
        self.shared
            .duration_ms
            .store(meta.duration.as_millis() as u64, Ordering::Relaxed);
        // A resume restore seeks this track to its exact saved position; any
        // other track (or a stale target) starts at 0. Streams can't seek.
        let mut start_ms = 0u64;
        if let Some((idx, pos)) = self.pending_seek.take() {
            if seekable && idx == self.index && !pos.is_zero() {
                dec.seek(pos);
                start_ms = pos.as_millis() as u64;
            }
        }
        self.decoded_us = start_ms * 1000;
        self.shared.decode_pos_ms.store(start_ms, Ordering::Relaxed);
        self.shared.index.store(self.index, Ordering::Relaxed);
        apply_replaygain_track(&mut self.dsp, &meta);
        *self.shared.meta.lock().unwrap() = Some(meta);
        self.decoder = Some(dec);
        // Track change (or resume) — refresh the resume index.
        self.save_resume();
        true
    }

    fn reset_current(&mut self) {
        if let Some(session) = self.managed.take() {
            session.cancel();
            drop(self.decoder.take());
            session.decoder_joined();
            let mut active = self.shared.managed.lock().unwrap();
            if active
                .as_ref()
                .is_some_and(|s| s.generation() == session.generation())
            {
                *active = None;
            }
        }
        self.decoder = None;
        self.decoded_us = 0;
        self.current_source = None; // drop any HTTP temp cache
        #[cfg(feature = "http")]
        {
            self.current_icy = None;
        }
        self.dsp.flush();
        self.shared.ring.lock().unwrap().clear();
    }

    /// Reflect a live stream's changing ICY `StreamTitle` (and the decoded
    /// sample rate, which is only known once decoding starts) into the shared
    /// metadata so `status()` shows the current song. Cheap no-op when nothing
    /// changed or there's no live stream.
    #[cfg(feature = "http")]
    fn refresh_icy(&mut self) {
        let rate = self.input_rate;
        let Some(icy) = self.current_icy.as_mut() else {
            return;
        };
        let current = icy.title.lock().unwrap().clone();
        if current == icy.last_title && icy.base.sample_rate == rate {
            return;
        }
        icy.last_title = current.clone();
        icy.base.sample_rate = rate; // 0 until the first chunk decodes
        let mut meta = icy.base.clone();
        if let Some(t) = current {
            // "Artist - Title" is the common StreamTitle form.
            match t.split_once(" - ") {
                Some((artist, title)) => {
                    meta.artist = artist.trim().to_string();
                    meta.title = title.trim().to_string();
                }
                None => meta.title = t,
            }
        }
        *self.shared.meta.lock().unwrap() = Some(meta);
    }

    // ---- insertion (Rockbox playlist_insert_track semantics) ------------

    /// Insert `tracks` at the given [`InsertPosition`]. Delegates to the
    /// pure [`perform_insert`] model, then reconciles engine-side state
    /// (decoder/ring for `Replace`, the shared queue length).
    fn insert_tracks(&mut self, tracks: Vec<PathBuf>, position: InsertPosition) {
        let is_replace = position == InsertPosition::Replace;
        perform_insert(
            &mut self.queue,
            &mut self.index,
            &mut self.last_insert_pos,
            &mut self.rng_state,
            self.playing,
            tracks,
            position,
        );
        if is_replace {
            // Cue the new queue from the top; the run loop reopens index 0.
            self.finishing = false;
            self.pending_seek = None;
            self.reset_current();
        }
        self.sync_queue();
    }

    /// Remove the track at `at` (Rockbox `playlist_delete`). Delegates the
    /// queue/index bookkeeping to the pure [`perform_remove`] model, then
    /// reconciles engine-side state: an emptied queue stops playback, and
    /// dropping the currently-playing track hard-cuts to whatever slid into
    /// its slot (a crossfade would need the track we just removed).
    fn remove_track(&mut self, at: usize) {
        if at >= self.queue.len() {
            return;
        }
        let removed_current = perform_remove(
            &mut self.queue,
            &mut self.index,
            &mut self.last_insert_pos,
            at,
        );
        if self.queue.is_empty() {
            self.clear_queue();
            return;
        }
        if removed_current {
            self.finishing = false;
            self.pending_seek = None;
            self.reset_current();
        }
        self.sync_queue();
    }

    /// Empty the queue and stop playback, mirroring [`Engine::stop_playback`]'s
    /// shared-state teardown. Also deletes the resume file — a cleared queue
    /// has nothing to resume.
    fn clear_queue(&mut self) {
        self.queue.clear();
        self.index = 0;
        self.last_insert_pos = None;
        self.pending_seek = None;
        self.finishing = false;
        self.playing = false;
        self.paused = false;
        self.reset_current();
        self.clear_resume();
        self.set_state(ST_STOPPED);
        self.shared
            .target_amp
            .store(0f32.to_bits(), Ordering::Relaxed);
        self.shared.decode_pos_ms.store(0, Ordering::Relaxed);
        self.shared.index.store(usize::MAX, Ordering::Relaxed);
        self.sync_queue();
    }

    // ---- resume persistence ---------------------------------------------

    /// Mirror the queue (len + contents) into `Shared` for the handle.
    fn sync_queue(&mut self) {
        self.rebuild_play_order();
        self.shared
            .queue_len
            .store(self.queue.len(), Ordering::Relaxed);
        *self.shared.queue.lock().unwrap() = self.queue.clone();
    }

    /// The playback position within the current track (decode position minus
    /// the ring-buffer lag) — what the listener has actually heard.
    fn playback_pos_ms(&self) -> u64 {
        let decode = self.shared.decode_pos_ms.load(Ordering::Relaxed);
        let rate = self.cfg.output_rate.max(1) as u64;
        let lag = (self.shared.ring_frames() as u64 * 1000) / rate;
        decode.saturating_sub(lag)
    }

    /// Write the resume file (queue + current index + exact position). No-op
    /// when resume is disabled or the queue is empty.
    fn save_resume(&mut self) {
        let Some(path) = self.cfg.resume_file.clone() else {
            return;
        };
        self.last_save = Instant::now();
        if self.queue.is_empty() {
            return;
        }
        let state = ResumeState {
            tracks: self.queue.clone(),
            index: self.index.min(self.queue.len() - 1),
            elapsed: Duration::from_millis(self.playback_pos_ms()),
        };
        let _ = resume::save(&path, &state);
    }

    /// Delete the resume file (queue finished — nothing to resume).
    fn clear_resume(&self) {
        if let Some(path) = &self.cfg.resume_file {
            resume::clear(path);
        }
    }

    /// The index the next auto/manual transition moves to, if any. Honors the
    /// play order (shuffle), repeat mode, and a pending manual-skip target.
    fn next_index(&self, auto: bool) -> Option<usize> {
        if let Some(t) = self.pending_manual_target {
            return Some(t);
        }
        // Repeat-one replays the current track on *automatic* advance only; a
        // manual `next` still steps to the following track.
        if auto && self.repeat == RepeatMode::One {
            return Some(self.index);
        }
        next_in_order(&self.order, self.index, self.repeat, true)
    }

    /// Rebuild [`Engine::order`] to match the current queue and shuffle state.
    /// Identity when shuffle is off; when on, the current track stays first
    /// (so toggling shuffle never interrupts it) and the remaining indices are
    /// Fisher-Yates shuffled with the engine RNG.
    fn rebuild_play_order(&mut self) {
        let len = self.queue.len();
        self.order = if self.shuffle {
            shuffled_order(len, self.index, &mut self.rng_state)
        } else {
            (0..len).collect()
        };
    }

    /// Publish shuffle + repeat state into [`Shared`] for the handle.
    fn sync_modes(&self) {
        self.shared.shuffle.store(self.shuffle, Ordering::Relaxed);
        self.shared
            .repeat
            .store(self.repeat.to_u8(), Ordering::Relaxed);
    }

    /// Move to the next track for playback. Returns false at end of queue.
    fn advance_index(&mut self, auto: bool) -> bool {
        match self.next_index(auto) {
            Some(n) => {
                self.index = n;
                self.decoder = None;
                true
            }
            None => false,
        }
    }

    fn set_state(&self, s: u8) {
        self.shared.state.store(s, Ordering::Relaxed);
    }
}

/// Pure Rockbox `playlist_insert_track` model: mutate `queue` / `index` /
/// `last_insert_pos` in place for `position`. `started` means playback has
/// begun (so the current track under `index` must be kept playing).
///
/// Ordered positions ([`InsertPosition::Prepend`], `Insert`, `InsertNext`,
/// `InsertLast`, `Index`) place the tracks as a contiguous, order-preserving
/// block. The shuffled positions place each track at its own random point;
/// `Replace` erases the queue and cues the new tracks from the top.
///
/// Kept free of engine/audio state so it is unit-testable in isolation.
fn perform_insert(
    queue: &mut Vec<PathBuf>,
    index: &mut usize,
    last_insert_pos: &mut Option<usize>,
    rng: &mut u64,
    started: bool,
    tracks: Vec<PathBuf>,
    position: InsertPosition,
) {
    if tracks.is_empty() && position != InsertPosition::Replace {
        return;
    }

    match position {
        InsertPosition::Replace => {
            *queue = tracks;
            *index = 0;
            *last_insert_pos = None;
        }
        InsertPosition::InsertShuffled => {
            for t in tracks {
                let amount = queue.len();
                let at = if started && amount > 0 {
                    // Random point between the current track and the end.
                    let offset = rand_below(rng, amount - *index);
                    (*index + offset + 1).min(amount)
                } else {
                    rand_below(rng, amount + 1)
                };
                insert_one(queue, index, last_insert_pos, started, at, t, false);
            }
        }
        InsertPosition::InsertLastShuffled => {
            // Freeze the region start, then drop each new track at a random
            // point within the growing tail so the batch ends up shuffled
            // among itself while earlier tracks stay put.
            let start = queue.len();
            for t in tracks {
                let span = queue.len() - start + 1;
                let at = start + rand_below(rng, span);
                insert_one(queue, index, last_insert_pos, started, at, t, false);
            }
        }
        _ => {
            let (mut at, sets_last) = block_start(queue, *index, *last_insert_pos, position);
            for t in tracks {
                insert_one(queue, index, last_insert_pos, started, at, t, sets_last);
                at += 1;
            }
        }
    }
}

/// Insert one track at raw position `at`, keeping the currently-playing
/// track under `index` and shifting `last_insert_pos` if it moved.
fn insert_one(
    queue: &mut Vec<PathBuf>,
    index: &mut usize,
    last_insert_pos: &mut Option<usize>,
    started: bool,
    at: usize,
    path: PathBuf,
    sets_last: bool,
) {
    let at = at.min(queue.len());
    let pre_amount = queue.len();
    queue.insert(at, path);

    if started && pre_amount > 0 && at <= *index {
        *index += 1;
    }
    if let Some(lp) = *last_insert_pos {
        if at <= lp {
            *last_insert_pos = Some(lp + 1);
        }
    }
    if sets_last {
        *last_insert_pos = Some(at);
    }
}

/// Pure Rockbox `playlist_delete` model: remove the track at `at` from
/// `queue` and reconcile `index` / `last_insert_pos` in place. Returns `true`
/// when the removed track was the currently-playing one (`at == index`), so
/// the caller can re-cue playback. `at` is assumed in range (the caller
/// bounds-checks); index bookkeeping:
///
/// * `at < index` — a track before the current one went away, so the cursor
///   shifts back one to keep pointing at the same track.
/// * `at == index` — the current track is gone; the cursor stays put (the
///   next track slides into the slot) but is clamped to the new last index
///   when the tail was removed.
/// * `at > index` — the current track is unaffected.
///
/// Kept free of engine/audio state so it is unit-testable in isolation.
fn perform_remove(
    queue: &mut Vec<PathBuf>,
    index: &mut usize,
    last_insert_pos: &mut Option<usize>,
    at: usize,
) -> bool {
    let removed_current = at == *index;
    queue.remove(at);

    if at < *index {
        *index -= 1;
    } else if removed_current && *index >= queue.len() {
        *index = queue.len().saturating_sub(1);
    }

    // Keep the "insert here next" anchor consistent with the shifted queue.
    match *last_insert_pos {
        Some(lp) if at == lp => *last_insert_pos = None,
        Some(lp) if at < lp => *last_insert_pos = Some(lp - 1),
        _ => {}
    }

    removed_current
}

/// Resolve the start index for an ordered-block insertion and whether it
/// updates `last_insert_pos`.
fn block_start(
    queue: &[PathBuf],
    index: usize,
    last_insert_pos: Option<usize>,
    position: InsertPosition,
) -> (usize, bool) {
    let amount = queue.len();
    match position {
        InsertPosition::Prepend => (0, false),
        InsertPosition::InsertNext => (if amount > 0 { index + 1 } else { 0 }, true),
        InsertPosition::InsertLast => (amount, true),
        InsertPosition::Insert => {
            let p = match last_insert_pos {
                Some(lp) if lp < amount => lp + 1,
                _ if amount > 0 => index + 1,
                _ => 0,
            };
            (p, true)
        }
        InsertPosition::Index(i) => (i.min(amount), false),
        // Shuffled / Replace are handled by perform_insert directly.
        _ => (amount, false),
    }
}

/// Uniform random in `0..n` via xorshift64 (`0` when `n == 0`).
fn rand_below(rng: &mut u64, n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let mut x = *rng;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *rng = x;
    (x % n as u64) as usize
}

/// A shuffled play order for `len` queue indices that keeps `current` first
/// (so enabling shuffle never interrupts the playing track) and Fisher-Yates
/// shuffles the remainder with `rng`. Returns identity order for `len <= 1`.
/// Pure and side-effect free (aside from advancing `rng`) so it is testable.
fn shuffled_order(len: usize, current: usize, rng: &mut u64) -> Vec<usize> {
    if len <= 1 {
        return (0..len).collect();
    }
    let mut rest: Vec<usize> = (0..len).filter(|&i| i != current).collect();
    for i in (1..rest.len()).rev() {
        rest.swap(i, rand_below(rng, i + 1));
    }
    let mut order = Vec::with_capacity(len);
    if current < len {
        order.push(current);
    }
    order.extend(rest);
    order
}

/// Step from `current` (a queue index) to the neighbouring queue index in
/// `order` — the next one when `forward`, the previous otherwise. Wraps around
/// under [`RepeatMode::All`]; returns `None` at the corresponding end when
/// repeat is off. A `current` not found in `order` is treated as position 0.
/// Pure and side-effect free, so it can be unit-tested in isolation.
fn next_in_order(
    order: &[usize],
    current: usize,
    repeat: RepeatMode,
    forward: bool,
) -> Option<usize> {
    if order.is_empty() {
        return None;
    }
    let len = order.len();
    let pos = order.iter().position(|&i| i == current).unwrap_or(0);
    let next_pos = if forward {
        if pos + 1 < len {
            Some(pos + 1)
        } else if repeat == RepeatMode::All {
            Some(0)
        } else {
            None
        }
    } else if pos > 0 {
        Some(pos - 1)
    } else if repeat == RepeatMode::All {
        Some(len - 1)
    } else {
        None
    };
    next_pos.map(|p| order[p])
}

/// Seed the shuffled-insertion PRNG. xorshift64 must not start at 0, so a
/// non-zero fallback constant is used if the clock is unavailable.
fn seed_rng() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    nanos | 0x9E37_79B9_7F4A_7C15
}

fn apply_replaygain_mode(dsp: &mut rockbox_dsp::Dsp, rg: &ReplayGainConfig) {
    let mode = match rg.mode {
        ReplayGainMode::Off => rockbox_dsp::REPLAYGAIN_OFF,
        ReplayGainMode::Track => rockbox_dsp::REPLAYGAIN_TRACK,
        ReplayGainMode::Album => rockbox_dsp::REPLAYGAIN_ALBUM,
    };
    dsp.set_replaygain(mode, rg.prevent_clipping, rg.preamp_db);
}

/// Apply every DSP-chain stage (past ReplayGain) to the shared DSP
/// singleton. Called once at engine start; individual stages are updated
/// live by the corresponding `Command` handlers.
fn apply_dsp_settings(dsp: &mut rockbox_dsp::Dsp, s: &DspSettings) {
    apply_equalizer(dsp, &s.equalizer);
    apply_tone(dsp, s.tone);
    apply_crossfeed(dsp, s.crossfeed);
    apply_surround(dsp, s.surround);
    dsp.set_channel_config(s.channel_mode.to_raw());
    dsp.set_stereo_width(s.stereo_width);
    apply_bass_enhancement(dsp, s.bass_enhancement);
    dsp.afr_enable(s.fatigue_reduction);
    dsp.set_compressor(&s.compressor.to_raw());
    dsp.dither_enable(s.dither);
    dsp.set_pitch(s.pitch);
}

/// Update one band in a mirror `bands` vector, padding it up to
/// [`EQ_BANDS`] flat bands (at the standard frequencies) first so the stored
/// snapshot always has a full set once any band has been touched.
fn upsert_eq_band(bands: &mut Vec<EqBand>, band: usize, setting: EqBand) {
    if band >= EQ_BANDS {
        return;
    }
    for i in bands.len()..EQ_BANDS {
        bands.push(EqBand {
            cutoff_hz: EQ_BAND_FREQUENCIES[i],
            q: 1.0,
            gain_db: 0.0,
        });
    }
    bands[band] = setting;
}

fn apply_equalizer(dsp: &mut rockbox_dsp::Dsp, eq: &Equalizer) {
    dsp.set_eq_precut(eq.precut_db);
    for (i, band) in eq.bands.iter().take(EQ_BANDS).enumerate() {
        dsp.set_eq_band(i, band.cutoff_hz, band.q, band.gain_db);
    }
    dsp.eq_enable(eq.enabled);
}

fn apply_tone(dsp: &mut rockbox_dsp::Dsp, t: ToneControls) {
    // Cutoffs must be set before set_tone: its prescale step recomputes the
    // shelf coefficients.
    dsp.set_tone_cutoffs(t.bass_cutoff_hz, t.treble_cutoff_hz);
    dsp.set_tone(t.bass_db, t.treble_db);
}

fn apply_surround(dsp: &mut rockbox_dsp::Dsp, s: Surround) {
    dsp.set_surround(s.delay_ms, s.balance, s.cutoff_low_hz, s.cutoff_high_hz);
}

fn apply_crossfeed(dsp: &mut rockbox_dsp::Dsp, cf: Crossfeed) {
    // Set the type first so the custom cross-mix params update the filter
    // (they're a no-op unless the type is already `Custom`).
    dsp.set_crossfeed(cf.mode.to_raw());
    dsp.set_crossfeed_direct_gain(cf.direct_gain);
    dsp.set_crossfeed_cross_params(cf.cross_gain, cf.high_freq_gain, cf.high_freq_cutoff);
}

fn apply_bass_enhancement(dsp: &mut rockbox_dsp::Dsp, pbe: BassEnhancement) {
    // Pre-cut before enabling: `pbe_enable` recomputes the filter using the
    // current precut.
    dsp.set_pbe_precut(pbe.precut);
    dsp.pbe_enable(pbe.strength);
}

fn apply_replaygain_track(dsp: &mut rockbox_dsp::Dsp, meta: &Metadata) {
    let rg = &meta.replaygain;
    dsp.set_replaygain_gains_raw(
        replaygain_c_long(rg.raw_track_gain),
        replaygain_c_long(rg.raw_album_gain),
        replaygain_c_long(rg.raw_track_peak),
        replaygain_c_long(rg.raw_album_peak),
    );
}

// Metadata stores i64, but the DSP ABI uses C long (32-bit on Windows).
// Saturate out-of-range metadata instead of wrapping or panicking during playback.
fn replaygain_c_long(value: i64) -> std::os::raw::c_long {
    value.clamp(
        std::os::raw::c_long::MIN as i64,
        std::os::raw::c_long::MAX as i64,
    ) as std::os::raw::c_long
}

#[test]
fn replaygain_c_long_preserves_values_and_saturates() {
    assert_eq!(replaygain_c_long(0), 0);
    assert_eq!(replaygain_c_long(1 << 24), 1 << 24);
    assert_eq!(replaygain_c_long(i64::MIN), std::os::raw::c_long::MIN);
    assert_eq!(replaygain_c_long(i64::MAX), std::os::raw::c_long::MAX);
}

#[cfg(test)]
mod insertion_tests {
    //! Unit tests for the pure Rockbox insertion model ([`perform_insert`]).
    //! These need no audio device — they exercise the queue/index bookkeeping
    //! directly.

    use super::*;

    /// Build a queue of single-letter paths.
    fn q(names: &str) -> Vec<PathBuf> {
        names
            .chars()
            .map(|c| PathBuf::from(c.to_string()))
            .collect()
    }

    /// Render a queue back to a compact string for assertions.
    fn s(queue: &[PathBuf]) -> String {
        queue
            .iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect()
    }

    /// Run one insertion against a fresh state and return
    /// `(queue_string, index, last_insert_pos)`.
    fn run(
        start: &str,
        index: usize,
        last: Option<usize>,
        started: bool,
        tracks: &str,
        pos: InsertPosition,
    ) -> (String, usize, Option<usize>) {
        let mut queue = q(start);
        let mut idx = index;
        let mut lip = last;
        let mut rng = 0x1234_5678_9abc_def0u64;
        perform_insert(
            &mut queue,
            &mut idx,
            &mut lip,
            &mut rng,
            started,
            q(tracks),
            pos,
        );
        (s(&queue), idx, lip)
    }

    #[test]
    fn insert_next_after_current_shifts_nothing_before() {
        // Playing "B" (index 1) in A B C; InsertNext X → A B X C, index unchanged.
        let (queue, idx, lip) = run("ABC", 1, None, true, "X", InsertPosition::InsertNext);
        assert_eq!(queue, "ABXC");
        assert_eq!(idx, 1);
        assert_eq!(lip, Some(2));
    }

    #[test]
    fn insert_next_block_preserves_order() {
        let (queue, idx, _) = run("ABC", 0, None, true, "XY", InsertPosition::InsertNext);
        assert_eq!(queue, "AXYBC");
        assert_eq!(idx, 0);
    }

    #[test]
    fn insert_last_appends_in_order() {
        let (queue, idx, lip) = run("ABC", 1, None, true, "XY", InsertPosition::InsertLast);
        assert_eq!(queue, "ABCXY");
        assert_eq!(idx, 1);
        assert_eq!(lip, Some(4)); // last inserted position
    }

    #[test]
    fn prepend_before_current_shifts_index() {
        // Playing index 1; prepend at 0 pushes the current track to index 2.
        let (queue, idx, _) = run("ABC", 1, None, true, "X", InsertPosition::Prepend);
        assert_eq!(queue, "XABC");
        assert_eq!(idx, 2);
    }

    #[test]
    fn prepend_when_stopped_keeps_index() {
        // Not started: index does not chase the insertion.
        let (queue, idx, _) = run("ABC", 0, None, false, "X", InsertPosition::Prepend);
        assert_eq!(queue, "XABC");
        assert_eq!(idx, 0);
    }

    #[test]
    fn insert_chaining_grows_one_block_after_current() {
        // Two separate Insert calls should build a contiguous block right
        // after the current track, in call order.
        let mut queue = q("ABC");
        let mut idx = 0usize;
        let mut lip = None;
        let mut rng = 1u64;
        perform_insert(
            &mut queue,
            &mut idx,
            &mut lip,
            &mut rng,
            true,
            q("X"),
            InsertPosition::Insert,
        );
        perform_insert(
            &mut queue,
            &mut idx,
            &mut lip,
            &mut rng,
            true,
            q("Y"),
            InsertPosition::Insert,
        );
        assert_eq!(s(&queue), "AXYBC");
        assert_eq!(idx, 0);
    }

    #[test]
    fn explicit_index_inserts_and_clamps() {
        let (queue, _, _) = run("ABC", 0, None, true, "X", InsertPosition::Index(2));
        assert_eq!(queue, "ABXC");
        let (clamped, _, _) = run("ABC", 0, None, true, "X", InsertPosition::Index(99));
        assert_eq!(clamped, "ABCX");
    }

    #[test]
    fn replace_erases_and_cues_from_top() {
        let (queue, idx, lip) = run("ABC", 2, Some(1), true, "XYZ", InsertPosition::Replace);
        assert_eq!(queue, "XYZ");
        assert_eq!(idx, 0);
        assert_eq!(lip, None);
    }

    /// Run one removal against a fresh state and return
    /// `(queue_string, index, last_insert_pos, removed_current)`.
    fn rm(
        start: &str,
        index: usize,
        last: Option<usize>,
        at: usize,
    ) -> (String, usize, Option<usize>, bool) {
        let mut queue = q(start);
        let mut idx = index;
        let mut lip = last;
        let removed_current = perform_remove(&mut queue, &mut idx, &mut lip, at);
        (s(&queue), idx, lip, removed_current)
    }

    #[test]
    fn remove_before_current_shifts_cursor_back() {
        // Playing "C" (index 2) in A B C D; delete "A" → B C D, still on "C".
        let (queue, idx, _, cur) = rm("ABCD", 2, None, 0);
        assert_eq!(queue, "BCD");
        assert_eq!(idx, 1);
        assert!(!cur);
    }

    #[test]
    fn remove_after_current_leaves_cursor() {
        // Playing "B" (index 1); delete "D" (index 3) → A B C, still on "B".
        let (queue, idx, _, cur) = rm("ABCD", 1, None, 3);
        assert_eq!(queue, "ABC");
        assert_eq!(idx, 1);
        assert!(!cur);
    }

    #[test]
    fn remove_current_keeps_index_so_next_track_slides_in() {
        // Playing "B" (index 1); delete "B" → A C D, index 1 now points at "C".
        let (queue, idx, _, cur) = rm("ABCD", 1, None, 1);
        assert_eq!(queue, "ACD");
        assert_eq!(idx, 1);
        assert!(cur);
    }

    #[test]
    fn remove_current_at_tail_clamps_to_new_last() {
        // Playing the final track "D" (index 3); delete it → A B C, cursor
        // clamps to the new last index 2.
        let (queue, idx, _, cur) = rm("ABCD", 3, None, 3);
        assert_eq!(queue, "ABC");
        assert_eq!(idx, 2);
        assert!(cur);
    }

    #[test]
    fn remove_last_remaining_track_empties_queue() {
        let (queue, idx, _, cur) = rm("A", 0, None, 0);
        assert_eq!(queue, "");
        assert_eq!(idx, 0);
        assert!(cur);
    }

    #[test]
    fn remove_shifts_insert_anchor() {
        // last_insert_pos tracks a slot; deleting before it shifts it back,
        // deleting it clears the anchor.
        let (_, _, lip, _) = rm("ABCD", 0, Some(2), 1);
        assert_eq!(lip, Some(1));
        let (_, _, lip, _) = rm("ABCD", 0, Some(2), 2);
        assert_eq!(lip, None);
        let (_, _, lip, _) = rm("ABCD", 0, Some(2), 3);
        assert_eq!(lip, Some(2)); // deletion after the anchor leaves it put
    }

    #[test]
    fn insert_shuffled_stays_after_current_and_keeps_it() {
        // Every random placement must land strictly after the current track
        // and leave the played prefix intact.
        for rng_seed in 1..200u64 {
            let mut queue = q("ABCDE");
            let mut idx = 2usize; // playing "C"
            let mut lip = None;
            let mut rng = rng_seed;
            perform_insert(
                &mut queue,
                &mut idx,
                &mut lip,
                &mut rng,
                true,
                q("X"),
                InsertPosition::InsertShuffled,
            );
            assert_eq!(queue.len(), 6);
            let xpos = queue.iter().position(|p| p == &PathBuf::from("X")).unwrap();
            assert!(xpos > idx, "X at {xpos} must be after current index {idx}");
            // Prefix up to and including the current track is untouched.
            assert_eq!(s(&queue[..=idx]), "ABC");
        }
    }

    #[test]
    fn insert_last_shuffled_keeps_prefix_and_adds_batch() {
        for rng_seed in 1..200u64 {
            let mut queue = q("ABC");
            let mut idx = 1usize;
            let mut lip = None;
            let mut rng = rng_seed;
            perform_insert(
                &mut queue,
                &mut idx,
                &mut lip,
                &mut rng,
                true,
                q("XYZ"),
                InsertPosition::InsertLastShuffled,
            );
            // Original tracks stay in their original order and positions.
            assert_eq!(s(&queue[..3]), "ABC");
            assert_eq!(queue.len(), 6);
            // The batch is entirely in the tail region.
            let tail = s(&queue[3..]);
            for c in ['X', 'Y', 'Z'] {
                assert!(
                    tail.contains(c),
                    "batch member {c} missing from tail {tail}"
                );
            }
        }
    }

    #[test]
    fn empty_insert_is_a_noop_except_replace() {
        let (queue, idx, _) = run("ABC", 1, None, true, "", InsertPosition::InsertLast);
        assert_eq!(queue, "ABC");
        assert_eq!(idx, 1);
        // Replace with no tracks clears the queue.
        let (empty, ridx, _) = run("ABC", 1, None, true, "", InsertPosition::Replace);
        assert_eq!(empty, "");
        assert_eq!(ridx, 0);
    }
}

#[cfg(test)]
mod dsp_tests {
    //! Pure tests for the DSP config helpers — no audio device required.

    use super::*;

    #[test]
    fn preset_builds_full_enabled_equalizer() {
        let eq = EqPreset::Rock.equalizer();
        assert!(eq.enabled);
        assert_eq!(eq.bands.len(), EQ_BANDS);
        // Center frequencies come from the standard table.
        assert_eq!(eq.bands[0].cutoff_hz, EQ_BAND_FREQUENCIES[0]);
        assert_eq!(
            eq.bands[EQ_BANDS - 1].cutoff_hz,
            EQ_BAND_FREQUENCIES[EQ_BANDS - 1]
        );
        // Precut equals the largest positive gain so boosts don't clip.
        let max_gain = EqPreset::Rock
            .gains()
            .iter()
            .cloned()
            .fold(0.0_f32, f32::max);
        assert_eq!(eq.precut_db, max_gain);
    }

    #[test]
    fn flat_preset_is_transparent_but_enabled() {
        let eq = EqPreset::Flat.equalizer();
        assert!(eq.enabled);
        assert_eq!(eq.precut_db, 0.0);
        assert!(eq.bands.iter().all(|b| b.gain_db == 0.0));
    }

    #[test]
    fn all_presets_have_names_and_ten_gains() {
        for p in EqPreset::ALL {
            assert!(!p.name().is_empty());
            assert_eq!(p.gains().len(), EQ_BANDS);
        }
    }

    #[test]
    fn upsert_pads_to_full_band_set() {
        let mut bands = Vec::new();
        upsert_eq_band(
            &mut bands,
            4,
            EqBand {
                cutoff_hz: 500,
                q: 2.0,
                gain_db: -3.0,
            },
        );
        // Padded up to a full set, with band 4 taking the new value.
        assert_eq!(bands.len(), EQ_BANDS);
        assert_eq!(bands[4].gain_db, -3.0);
        assert_eq!(bands[4].q, 2.0);
        // Untouched bands are flat at their standard frequency.
        assert_eq!(bands[0].gain_db, 0.0);
        assert_eq!(bands[0].cutoff_hz, EQ_BAND_FREQUENCIES[0]);
    }

    #[test]
    fn dsp_settings_default_is_neutral() {
        let s = DspSettings::default();
        assert!(!s.equalizer.enabled);
        assert_eq!(s.stereo_width, 100);
        assert_eq!(s.pitch, PITCH_NORMAL);
        assert_eq!(s.channel_mode, ChannelMode::Stereo);
        assert_eq!(s.compressor.threshold_db, 0);
        // Crossfeed / PBE / AFR default to disabled.
        assert_eq!(s.crossfeed.mode, CrossfeedMode::Off);
        assert_eq!(s.bass_enhancement.strength, 0);
        assert_eq!(s.fatigue_reduction, 0);
    }

    #[test]
    fn crossfeed_mode_maps_to_rockbox_ids() {
        assert_eq!(CrossfeedMode::Off.to_raw(), rockbox_dsp::CROSSFEED_OFF);
        assert_eq!(CrossfeedMode::Meier.to_raw(), rockbox_dsp::CROSSFEED_MEIER);
        assert_eq!(
            CrossfeedMode::Custom.to_raw(),
            rockbox_dsp::CROSSFEED_CUSTOM
        );
    }

    #[test]
    fn crossfeed_defaults_mirror_rockbox() {
        // Rockbox's own defaults (settings_list.c).
        let cf = Crossfeed::default();
        assert_eq!(cf.direct_gain, -15);
        assert_eq!(cf.cross_gain, -60);
        assert_eq!(cf.high_freq_gain, -160);
        assert_eq!(cf.high_freq_cutoff, 700);
    }
}

#[cfg(test)]
mod shuffle_repeat_tests {
    //! Pure tests for the shuffle/repeat play-order logic — no audio device.

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn repeat_off_stops_at_both_ends() {
        let order = vec![0, 1, 2];
        assert_eq!(next_in_order(&order, 2, RepeatMode::Off, true), None); // past last
        assert_eq!(next_in_order(&order, 0, RepeatMode::Off, false), None); // before first
        assert_eq!(next_in_order(&order, 1, RepeatMode::Off, true), Some(2));
        assert_eq!(next_in_order(&order, 1, RepeatMode::Off, false), Some(0));
    }

    #[test]
    fn repeat_all_wraps_both_directions() {
        let order = vec![0, 1, 2];
        assert_eq!(next_in_order(&order, 2, RepeatMode::All, true), Some(0));
        assert_eq!(next_in_order(&order, 0, RepeatMode::All, false), Some(2));
    }

    #[test]
    fn follows_play_order_not_queue_order() {
        // Shuffled order: play 2, then 0, then 1.
        let order = vec![2, 0, 1];
        assert_eq!(next_in_order(&order, 2, RepeatMode::Off, true), Some(0));
        assert_eq!(next_in_order(&order, 0, RepeatMode::Off, true), Some(1));
        assert_eq!(next_in_order(&order, 1, RepeatMode::Off, true), None);
        assert_eq!(next_in_order(&order, 1, RepeatMode::All, true), Some(2)); // wrap to first
    }

    #[test]
    fn empty_order_never_advances() {
        assert_eq!(next_in_order(&[], 0, RepeatMode::All, true), None);
        assert_eq!(next_in_order(&[], 5, RepeatMode::Off, false), None);
    }

    #[test]
    fn forward_cycle_under_repeat_all_visits_each_once_then_wraps() {
        let order = vec![3, 1, 0, 2];
        let mut cur = order[0];
        let mut visited = vec![cur];
        for _ in 1..order.len() {
            cur = next_in_order(&order, cur, RepeatMode::All, true).unwrap();
            visited.push(cur);
        }
        // After the last, forward wraps back to the first.
        assert_eq!(
            next_in_order(&order, cur, RepeatMode::All, true),
            Some(order[0])
        );
        visited.sort();
        assert_eq!(visited, vec![0, 1, 2, 3]);
    }

    #[test]
    fn repeat_mode_u8_roundtrips() {
        for m in [RepeatMode::Off, RepeatMode::One, RepeatMode::All] {
            assert_eq!(RepeatMode::from_u8(m.to_u8()), m);
        }
        assert_eq!(RepeatMode::from_u8(200), RepeatMode::Off); // unknown → Off
    }

    #[test]
    fn shuffled_order_is_a_permutation_with_current_first() {
        let mut rng = 0x1234_5678_9abc_def0u64;
        for len in 1..12usize {
            for current in 0..len {
                let order = shuffled_order(len, current, &mut rng);
                assert_eq!(order.len(), len, "len {len}");
                assert_eq!(order[0], current, "current stays first (len {len})");
                let set: HashSet<usize> = order.iter().copied().collect();
                assert_eq!(set, (0..len).collect(), "must be a permutation (len {len})");
            }
        }
    }

    #[test]
    fn shuffle_actually_reorders_for_larger_queues() {
        // With a decent queue size, at least one shuffle must differ from
        // identity (guards against an accidental no-op shuffle).
        let mut rng = 42u64;
        let reordered =
            (0..8).any(|_| shuffled_order(10, 0, &mut rng) != (0..10).collect::<Vec<_>>());
        assert!(reordered, "shuffle never reordered a 10-track queue");
    }
}

#[cfg(test)]
mod level_meter_tests {
    use super::*;

    fn drive(meter: &mut LevelMeter, alpha: f32, sample: f32, frames: usize) {
        let band_alphas = LevelMeter::band_alphas(44_100);
        for _ in 0..frames {
            meter.push(alpha, &band_alphas, sample, sample, sample);
        }
    }

    /// One second of a pure tone; returns the band index holding the peak.
    fn dominant_band(hz: f32) -> usize {
        let rate = 44_100u32;
        let alpha = LevelMeter::alpha(rate);
        let band_alphas = LevelMeter::band_alphas(rate);
        let mut meter = LevelMeter::default();
        for i in 0..rate {
            let t = i as f32 / rate as f32;
            let sample = (std::f32::consts::TAU * hz * t).sin() * 0.8;
            meter.push(alpha, &band_alphas, sample.abs(), sample.abs(), sample);
        }
        let levels = meter.take();
        levels
            .bands
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .map(|(i, _)| i)
            .unwrap()
    }

    /// The ladder is a spectrum, not a shape: a bass tone peaks in a low band,
    /// a treble tone in a high one, and they must not land anywhere near each
    /// other. Exact bins are not asserted — one-pole skirts are wide — only
    /// the ordering that makes a visualiser honest.
    #[test]
    fn bands_separate_bass_from_treble() {
        let low = dominant_band(60.0);
        let mid = dominant_band(1_000.0);
        let high = dominant_band(8_000.0);
        assert!(low < mid, "60 Hz in band {low}, 1 kHz in band {mid}");
        assert!(mid < high, "1 kHz in band {mid}, 8 kHz in band {high}");
        assert!(low <= 2, "60 Hz should land at the bottom, got {low}");
        assert!(high >= 10, "8 kHz should land near the top, got {high}");
    }

    /// Silence reads as silence, not as the last thing that played.
    #[test]
    fn silence_is_zero() {
        let mut meter = LevelMeter::default();
        drive(&mut meter, LevelMeter::alpha(44_100), 0.0, 512);
        assert_eq!(meter.take(), Levels::default());
    }

    /// A constant signal is entirely below 200 Hz, so both bands see it.
    #[test]
    fn a_steady_signal_fills_both_bands() {
        let mut meter = LevelMeter::default();
        let alpha = LevelMeter::alpha(44_100);
        // Long enough for the one-pole to settle.
        drive(&mut meter, alpha, 0.5, 44_100);

        let levels = meter.take();
        assert!((levels.left - 0.5).abs() < 0.01, "{levels:?}");
        assert!(
            levels.low_left > 0.9,
            "bass should reach the top: {levels:?}"
        );
    }

    /// The point of the low band: something that alternates every sample is
    /// the highest frequency representable, and must not move a bass meter.
    #[test]
    fn the_low_band_ignores_high_frequencies() {
        let mut meter = LevelMeter::default();
        let alpha = LevelMeter::alpha(44_100);
        let band_alphas = LevelMeter::band_alphas(44_100);
        for i in 0..44_100 {
            let sample = if i % 2 == 0 { 0.9 } else { -0.9 };
            meter.push(alpha, &band_alphas, sample, sample, sample);
        }

        let levels = meter.take();
        assert!(levels.left > 0.8, "full band should see it: {levels:?}");
        assert!(levels.low_left < 0.1, "bass band should not: {levels:?}");
    }

    /// Taking resets the window but keeps the filter, so the next buffer does
    /// not start from a re-settling low-pass.
    #[test]
    fn taking_resets_the_window_but_not_the_filter() {
        let mut meter = LevelMeter::default();
        let alpha = LevelMeter::alpha(44_100);
        drive(&mut meter, alpha, 0.5, 44_100);
        let settled = meter.take().low_left;

        drive(&mut meter, alpha, 0.5, 64);
        let next = meter.take().low_left;
        assert!(
            (next - settled).abs() < 0.05,
            "the filter restarted: {settled} then {next}"
        );
    }

    #[test]
    fn levels_never_exceed_one() {
        let mut meter = LevelMeter::default();
        drive(&mut meter, LevelMeter::alpha(44_100), 1.0, 4_096);
        let levels = meter.take();
        assert!(levels.left <= 1.0 && levels.low_left <= 1.0, "{levels:?}");
    }
}

#[cfg(test)]
mod output_tests;
