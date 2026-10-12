#!/usr/bin/env bash
# Headless playback tests still need a real, continuously drained output.
set -euo pipefail

fifo_dir=$(mktemp -d "${RUNNER_TEMP:-${TMPDIR:-/tmp}}/music-player-tests.XXXXXX")
drain_pid=
cleanup() {
  if [[ -n "$drain_pid" ]]; then
    kill "$drain_pid" 2>/dev/null || true
    wait "$drain_pid" 2>/dev/null || true
  fi
  exec 9>&-
  rm -f -- "$fifo_dir/audio.pcm"
  rmdir -- "$fifo_dir"
}
trap cleanup EXIT

mkfifo "$fifo_dir/audio.pcm"
# Keep a writer open between Player instances, so the drain never sees EOF.
exec 9<>"$fifo_dir/audio.pcm"
cat <"$fifo_dir/audio.pcm" >/dev/null &
drain_pid=$!
export MUSIC_PLAYER_AUDIO_OUTPUT="fifo:$fifo_dir/audio.pcm"

# Preserve the coverage command's failure and clean up after it in either case.
"$@"
kill -0 "$drain_pid"
