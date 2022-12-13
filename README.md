## Music Player (written in Rust)

<p>
  <a href="LICENSE" target="_blank">
    <img alt="License: MIT" src="https://img.shields.io/badge/License-MIT-blue.svg" />
  </a>
  <a href="https://buf.build/tsiry/musicserverapis/docs/main:music.v1alpha1">
    <img src="https://img.shields.io/badge/apidocs-yes-cyan.svg" />
  </a>
  <a href="https://crates.io/crates/music-player" target="_blank">
    <img src="https://img.shields.io/crates/v/music-player.svg" />
  </a>
   <a href="https://crates.io/crates/music-player" target="_blank">
    <img src="https://img.shields.io/crates/dr/music-player" />
  </a>
  <a href="https://codecov.io/gh/tsirysndr/music-player" target="_blank">
    <img src="https://codecov.io/gh/tsirysndr/music-player/branch/chore/tests/graph/badge.svg?token=" />
  </a>
  <a href="https://github.com/tsirysndr/music-player/actions/workflows/ci.yml" target="_blank">
    <img src="https://github.com/tsirysndr/music-player/actions/workflows/ci.yml/badge.svg" />
  </a>
   <a href="https://github.com/tsirysndr/music-player/actions/workflows/release.yml" target="_blank">
    <img alt="release" src="https://github.com/tsirysndr/music-player/actions/workflows/release.yml/badge.svg" />
  </a>
  <a href="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml" target="_blank">
    <img alt="rust-clippy" src="https://github.com/tsirysndr/music-player/actions/workflows/rust-clippy.yml/badge.svg?branch=master" />
  </a>
  <a href="https://feat-webui--6343b23f7b47cd6de45a5849.chromatic.com/" target="_blank">
  <img src="https://raw.githubusercontent.com/storybooks/brand/master/badge/badge-storybook.svg">
  </a>
  <a href="https://discord.gg/reJ9gUNsMV" target="_blank">
    <img alt="discord-server" src="https://img.shields.io/discord/1026789060515205181?label=discord&logo=discord&color=5865F2">
  </a>
</p>

<p style="margin-top: 20px; margin-bottom: 50px;">
<img src="./preview.png" width="100%" />
</p>

<p style="margin-top: 20px; margin-bottom: 50px;">
<img src="./preview-tui.png" width="100%" />
</p>

Note: This is a work in progress. 🏗️🚧

This is a simple music player that I made for my own use. It is written in Rust and uses [rodio](https://github.com/RustAudio/rodio), [symphonia](https://github.com/pdeljanov/Symphonia), and [gRPC](https://grpc.io/) libraries.<br />
Music is played through the server's audio device. The daemon stores info about all available music, and this info can be easily searched and retrieved.
Like [mpd](https://github.com/MusicPlayerDaemon/MPD) or [Mopidy](https://github.com/mopidy/mopidy) but written in Rust.

<p style="margin-top: 20px; margin-bottom: 20px;">
  <img src="./preview.svg" width="800" />
</p>

## Installation

```bash
# Install dependencies
brew install protobuf # macOS
sudo apt-get install -y libasound2-dev protobuf-compiler # Ubuntu/Debian
choco install protoc # Windows using Chocolatey Package Manager
# Compile
git clone https://github.com/tsirysndr/music-player.git
cd music-player/webui/musicplayer
nvm install # install node version specified in .nvmrc (optional on windows)
npm install -g yarn
yarn install && yarn build # build webui
cd ../..
cargo install --path .
```

Note: Don't forget to add `~/.cargo/bin` to your `PATH` environment variable.

### macOS/Linux

```bash
brew install tsirysndr/tap/musicplayer
```

Or download the latest release for your platform [here](https://github.com/tsirysndr/music-player/releases).

## 📦 Downloads

<!-- download start -->

**Latest (Desktop):**

- `Mac`: arm64: [music-player-desktop_v0.2.0-alpha.8_aarch64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player-desktop_v0.2.0-alpha.8_aarch64-apple-darwin.tar.gz) intel: [Music_Player_v0.2.0-alpha.8_x64.dmg](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/Music_Player_v0.2.0-alpha.8_x64.dmg)
- `Linux`: [music-player_v0.2.0-alpha.8_amd64.deb](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player_v0.2.0-alpha.8_amd64.deb)
- `Windows`: [Music_Player_x64_en-US.msi](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/Music_Player_x64_en-US.msi)

**Latest (CLI):**

- `Mac`: arm64: [music-player_v0.2.0-alpha.8_aarch64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player_v0.2.0-alpha.8_aarch64-apple-darwin.tar.gz) intel: [music-player_v0.2.0-alpha.8_x86_64-apple-darwin.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player_v0.2.0-alpha.8_x86_64-apple-darwin.tar.gz)
- `Linux`: [music-player_v0.2.0-alpha.8_x86_64-unknown-linux-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player_v0.2.0-alpha.8_x86_64-unknown-linux-gnu.tar.gz)
- `Windows`: [music-player_x86_64-pc-windows-gnu.tar.gz](https://github.com/tsirysndr/music-player/releases/download/v0.2.0-alpha.8/music-player_x86_64-pc-windows-gnu.tar.gz)

[Other version...](https://github.com/tsirysndr/music-player/releases)


## Start the server

```bash
music-player
```

## Usage

```
USAGE:
    music-player [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    albums      List all albums
    artists     List all artists
    help        Print this message or the help of the given subcommand(s)
    next        Play the next song
    pause       Pause the current song
    open        Open audio file
    playlist    Manage playlists
    prev        Play the previous song
    queue       Manage the queue
    play        Resume the current song
    scan        Scan music library: $HOME/Music
    search      Search for a song, album, artist or playlist
    stop        Stop the current song
    tracks      List all tracks
```

### GraphQL API

```bash
# Start the server
music-player
```

Open [http://localhost:5053/graphiql](http://localhost:5053/graphiql) in your browser.

<p style="margin-top: 20px; margin-bottom: 20px;">
 <img src="./preview-api.png" width="100%" />
</p>

### Features

- [x] Play music from specified path
- [x] Configuration file support
- [x] [gRPC API](https://buf.build/tsiry/musicserverapis/docs/main:music.v1alpha1) for controlling the player
- [x] Scan music library
- [x] Play/Pause/Stop music
- [x] Next/Previous track
- [x] Create/Delete playlists
- [x] Music Player Client
- [x] Terminal UI (using [tui-rs](https://github.com/fdehau/tui-rs))
- [x] GraphQL API 
- [x] Web UI
- [ ] Desktop version (using [gtk-rs](https://gtk-rs.org/))
- [ ] Android Library
- [ ] iOS Library
- [ ] Mobile version (React Native)
- [ ] Stream to Chromecast
- [ ] Stream to Airplay
- [ ] Stream to Kodi
- [ ] Stream from Youtube (audio only)
- [ ] Stream from Spotify
- [ ] Stream from Soundcloud
- [ ] Stream from Deezer
- [ ] Stream from DatPiff
- [ ] Stream from Tidal
- [ ] Stream from [MyVazo](https://myvazo.com/)
