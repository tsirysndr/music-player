# Public Player HTTP feasibility probe

This isolated package runs the released `rockbox-playback 0.7.0` through its public
Player API. It generates synthetic MP3 audio, serves only loopback HTTP, captures
TCP PCM delivery, and records stop/Drop behavior. No real media server, account,
token, application database or playback setting is used.

Build and execution run in GitHub Actions (`Emby engine feasibility`). The job
uses Ubuntu 24.04, Rust 1.96.0, FFmpeg and ALSA development libraries. The dependency
keeps its default features; the selected output is TCP, so no physical audio
device is required. The package has its own workspace/lock and does not modify
the application dependency graph.

The CI commands are:

```sh
cargo build --locked --manifest-path tools/emby-engine-probe/Cargo.toml
python3 tools/emby-engine-probe/run.py
python3 tools/emby-engine-probe/verify.py tools/emby-engine-probe/results
```

Cases: known-length response ignoring Range; unknown-length chunked MP3; delayed
headers; body stall; required authentication header; HTML disguised as audio;
short valid EOF. A 5-second watchdog after stop kills only the owned experiment
child when Drop hangs. That outcome is recorded as a cancellation failure, never
as successful cancellation. Fixture release after 15 seconds is a cleanup bound
for cancellation probes, not a test of the separate 30-second read-stall timeout.

The verifier checks complete evidence and positive controls. A successful CI job
means the experiment ran correctly. It does **not** mean product gates passed:
the public API cannot establish reader-task join or native consumption. TCP
delivery is not audible playback or FIFO downstream acknowledgment. Unknown
duration EOF is not proof of media completion. Full Emby integration, output
clock/generation changes, native/FIFO output, server cleanup and long playback
require separate implementation and acceptance.

Artifacts include the exact executable/source/lock identity, environment and
build log, synthetic input, per-case engine/HTTP/PCM observations, and summary.
No release asset or application package is published.
