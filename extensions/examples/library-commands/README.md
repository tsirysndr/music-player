# Library Reports

Three commands that read the library and hand back a summary.

| Command           | What it does                                            |
| ----------------- | ------------------------------------------------------- |
| `library_summary` | Counts of tracks, albums, artists, playlists and radios |
| `decades`         | How the library spreads across decades                  |
| `unplayed`        | Tracks that have never been played                      |

## What it shows

- **Declaring commands up front**, so the UI can list them before any runs.
- **Reading the library** through the host functions, with RSQL filters.
- **Returning both** a human line (`message`) and structured JSON (`data`), so
  a caller can display either.

This is the example to read for the library API — it touches tracks, albums,
artists, playlists and saved radios.

Needs `libraryRead = true`. Without it the queries return empty lists and the
reports all read zero.

## Build

```sh
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/plugin.wasm .
```
