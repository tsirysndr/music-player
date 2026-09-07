# LRCLIB Lyrics

Synced lyrics from [lrclib.net](https://lrclib.net) — free, and needs no key.

## What it shows

- **Answering only for what you know.** The host merges answers across
  providers, first non-empty per field, so leaving `artwork_url` and `bio`
  unset lets other providers fill them in.
- **Preferring synced over plain**, with a clean fallback.
- **"Not found" is an empty answer, not an error.** A 404 and a network failure
  are different things, and only one is worth logging.
- **Matching the right version.** Sending the album and duration is what makes
  LRCLIB pick the right cut of a song that exists in several.

## Build

```sh
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/plugin.wasm .
```
