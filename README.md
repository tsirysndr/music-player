## Music Player (written in Rust)

<p>
  <a href="LICENSE" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg" />
  </a>
  <a href="https://buf.build/tsiry/musicserverapis/docs/main:music.v1alpha1">
    <img src="https://img.shields.io/badge/apidocs-yes-cyan.svg" />
  </a>
  <a href="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml" target="_blank">
    <img alt="rust-clippy" src="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml/badge.svg?branch=master" />
  </a>
</p>

<p style="margin-top: 50px; margin-bottom: 50px;">
<img src="./cover.svg" height="300" />
</p>

Note: This is a work in progress.

This is a simple music player that I made for my own use. It is written in Rust and uses [rodio](https://github.com/RustAudio/rodio), [symphonia](https://github.com/pdeljanov/Symphonia), and [gRPC](https://grpc.io/) libraries.

## Installation

```bash
git clone https://github.com/tsirysndr/music-player.git
cd music-player
cargo install --path .
```

## Usage

```
USAGE:
    music-player [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help    Print this message or the help of the given subcommand(s)
    play    Play a song
```

### Features

- [x] Play music from secified path
- [x] Configuration file support
- [x] [gRPC API](https://buf.build/tsiry/musicserverapis/docs/main:music.v1alpha1) for controlling the player
- [x] Scan music library
- [x] Play/Pause/Stop music
- [ ] Next/Previous track
- [ ] Create/Delete playlists
- [ ] Web UI
- [ ] Desktop version (using [gtk-rs](https://gtk-rs.org/))
- [ ] Mobile version
- [ ] Stream to Chromecast
- [ ] Stream to Airplay
- [ ] Stream to Kodi
- [ ] Stream from Youtube (audio only)
- [ ] Stream from Spotify
- [ ] Stream from Soundcloud
- [ ] Stream from Deezer
- [ ] Stream from DatPiff
- [ ] Stream from [MyVazo](https://myvazo.com/)
