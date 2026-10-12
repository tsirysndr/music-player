# Patched dependencies

`rockbox-playback-0.7.0` is the crates.io source archive, SHA-256
`29c9f82837524c81bebf5c8590bfdce1f211f6aba0403ea79fe3593bb1c282a9`,
from https://crates.io/crates/rockbox-playback/0.7.0 (GPL-2.0-or-later).

The original source patch converts ReplayGain i64 metadata to the DSP's platform
C long type, saturating at its bounds. On Windows C long is 32-bit; upstream
0.7.0 fails to compile at this call. A boundary test covers preservation and
saturation. Remove this patch when an upstream release fixes the conversion.

The M1b patch adds a host-owned forward-only reader API to Player. Each stream
has its own generation and independent cancellation callback, reader release,
codec status and codec join observations. Stream EOF is EndUnconfirmed and
cannot trigger queue auto-advance; pause and replacement cancel the reader.
The callback must wake blocked IO and reader destruction must join the host
transport. Legacy URL handling remains on the upstream path.

This is an experimental integration under tools/emby-runtime-probe. The daemon
has not adopted it yet. FIFO and connected TCP/Unix writers now use nonblocking
writes. Cancellation revokes unsent PCM under the same gate as each final write;
partial frame cancellation is zero-padded before subsequent stereo frames.
Normal reader EOF releases input without revoking buffered output. Direct libc
use is Unix-only for O_NONBLOCK and FIFO type validation.

The FIFO experiment holds its reader open without consuming and requires actual
nonzero delivery and WouldBlock before Drop. Output counters include silence and
describe kernel acceptance, not confirmed receiver consumption. Existing stdout
remains blocking and is outside the managed cancellation guarantee; socket
connect/accept during construction is also still blocking. CPAL callback buffers
already handed to the device and bytes already accepted by a kernel/receiver
cannot be retracted by clearing the local ring.

StreamSession::output_snapshot records complete post-DSP media frames per
generation, excluding underrun silence, alignment padding and cancelled partial
frames. ByteStream means kernel acceptance; DeviceBuffer means CPAL callback
submission. Stdout is Unavailable. Output duration is not a source checkpoint:
the host still needs to map pitch/rate and account for receiver buffering.
Retained cancelled sessions keep their final count while newer sessions advance.

The public Player replacement fixture uses distinct 440/880 Hz sources, including
a source stalled before response headers. It checks B audio, cancellation of A,
late cancellation of A while B continues, and resource joins. FIFO evidence checks
that the session clock freezes during backpressure and survives Drop.
Confirmed consumption clocks, latest-intent coordination, TCP/Unix backpressure
and all real Emby acceptance remain separate M1b/M2/M6 gates.
