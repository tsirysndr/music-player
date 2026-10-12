# Managed Player runtime probe

Build and run only in GitHub Actions (Emby engine feasibility, runtime job).
This independent locked Cargo package uses the repository's patched Player and
host-owned transport. The sibling emby-engine-probe still uses registry 0.7.0
as a control. Both feed real MP3 through the public Player into a TCP collector.

The fixtures exercise finite/chunked input, blocked headers/body, header auth
positive and negative controls, fake audio, short EOF, pause, and Player drop.
verify.py recomputes cancellation latency from per-generation reader release,
codec join and transport worker exit events. A five-second process watchdog is
failure cleanup; the registered reader/codec cancellation threshold is two seconds.

Results retain raw events, PCM delivery counts, request facts, source hashes,
compiler environment and summaries. Only synthetic loopback credentials are used.
No private server is reached by CI. The transport owns bounded memory and never
materializes a media file. HTTP content length does not grant random access.

The output experiments keep real FIFO, TCP and Unix receivers open without
reading. Socket receivers listen with a small receive buffer; Player connects.
Each case requires nonzero PCM, actual WouldBlock and at least 300 ms of frozen
media/byte counters with increasing backpressure before requesting Drop. The
receiver descriptor remains open through process exit, including failure cleanup.
Drop, resource release and observed process exit each have a two-second gate;
the five-second watchdog only cleans up failures. All three cases run even if
one fails. Raw events, descriptor lifetime, hashes and the shared 120-second MP3
are retained in results/output-{fifo,tcp,unix}.json and tone-output-120s.mp3.
This checks connected/opened output, not blocking socket construction, stdout,
CPAL or receiver consumption.
TCP delivery, decoder elapsed time and ring drain are not proof of audible consumption. No automatic media completion is claimed;
short clean decoder EOF retains position as EndUnconfirmed. ServerOffset resume
requires a future host resolver to supply a fresh pinned stream and checkpoint.

The replacement binary and replacement.py exercise public Player A-to-B switches
using distinct 440/880 Hz MP3 sources. One case starts with confirmed A delivery;
the other holds A response headers until its resources have been reclaimed.
Both require fresh B audio within two seconds, sustained B after cancelling the
retained A handle again, and bounded Player Drop. Raw PCM, frequency windows,
requests, control events and hashes are saved alongside replacement.json even
on failure. The workflow builds both probe binaries before running the harness.

StreamSession output snapshots count complete post-DSP media frames per
generation. FIFO evidence checks a frozen clock under backpressure; replacement
evidence checks independent A/B clocks and retained A counts. Alignment padding,
underrun silence and cancelled partial frames are excluded; silent media counts.
These are output durations, not source checkpoints or receiver consumption.
