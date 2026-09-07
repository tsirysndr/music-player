# Mood & Era

Adds two terms to the smart-playlist vocabulary:

```
ext:mood==energetic;year>2015
ext:era==nineties,ext:era==eighties
ext:mood!=calm;playcount>5
```

| Predicate  | Values                                                 |
| ---------- | ------------------------------------------------------ |
| `ext:mood` | `energetic`, `calm`, `melancholy`, `upbeat`, `focused` |
| `ext:era`  | `sixties` … `twentytwenties`                           |

## What it shows

- **Declaring predicates with example values**, so a picker can offer them.
- **Honouring the operator.** `!=` is the complement of `==` — getting this
  wrong makes half the vocabulary silently do the wrong thing.
- **Keeping `evaluate` cheap.** It runs once per candidate track, so this one
  answers from the track it is handed and makes no request at all. A predicate
  that called an API per track would be unusable on a library of any size.

The mood is derived from genre keywords — a stand-in for the audio analysis or
service lookup a real one would do.

## Build

```sh
cargo build --release --target wasm32-wasip1
cp target/wasm32-wasip1/release/plugin.wasm .
```
