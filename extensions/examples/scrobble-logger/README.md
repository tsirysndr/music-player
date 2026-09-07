# Scrobble Logger

Posts every play to a webhook. The smallest useful `events` extension.

## Configure

```toml
[permissions.config]
webhookUrl = "https://api.listenbrainz.org/1/submit-listens"
token = "your-token"
```

Also add your endpoint's host to `allowedHosts` — the sandbox refuses any host
that is not listed.

## What it shows

- Handling only the events you care about (likes and scans are ignored).
- Reading configuration the manifest declared.
- Failing softly: a webhook being down logs a line and moves on, rather than
  interrupting playback.

## Build

```sh
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/plugin.wasm .
```
