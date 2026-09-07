# Examples

One worked example per capability. Each is a complete, buildable extension —
copy one and edit.

| Example                                | Capability   | Shows                                                                                        |
| -------------------------------------- | ------------ | -------------------------------------------------------------------------------------------- |
| [`scrobble-logger`](scrobble-logger)   | `events`     | Handling only the events you care about; config; failing softly when a webhook is down       |
| [`lyrics-provider`](lyrics-provider)   | `metadata`   | Answering only for what you know; synced-over-plain fallback; "not found" as an empty answer |
| [`library-commands`](library-commands) | `commands`   | Declaring commands up front; reading the whole library with RSQL; human + structured results |
| [`mood-predicate`](mood-predicate)     | `predicates` | Adding `ext:mood` / `ext:era`; honouring `!=`; keeping per-track evaluation cheap            |
| [`radio-source`](radio-source)         | `source`     | A full media source: browse, search, resolve streams at play time, call an allow-listed host |

## Build them all

```sh
for ex in scrobble-logger lyrics-provider library-commands mood-predicate radio-source; do
  (cd "$ex" && cargo build --release --target wasm32-wasip1 \
     && cp target/wasm32-wasip1/release/plugin.wasm .)
done
```

The committed `plugin.wasm` files are what the host's integration tests load, so
rebuild them after changing an example.

## Try one

```sh
# `extension list` prints the search paths; the first is where to copy to.
music-player extension list

DEST="$HOME/Library/Application Support/music-player/extensions/com.example.mood-predicate"
mkdir -p "$DEST" && cp mood-predicate/plugin.{toml,wasm} "$DEST/"
music-player extension list   # now shows it as loaded
```

## Note on the stubs

Each example ends with a block of stub functions for the capabilities it does
*not* declare. `pdk.rs` is generated from the whole schema, so every module
exports every function in it; the stubs satisfy the linker. They are never
called — the manifest is what decides which exports the host invokes.
