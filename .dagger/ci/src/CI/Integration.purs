module CI.Integration where

import Prelude
import CI.Base as Base
import CI.Rust as Rust
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Types (File)
import Effect.Aff (Aff)

-- The former standalone integration job: migrate, scan, start the daemon, then
-- test every library with all features. Keep this separate from workspace CI.
test :: Client -> Rust.Input -> Aff File
test client input = do
  arch <- Base.platform input.platform
  pure $ Rust.prepared client arch (input.platform <> "-integration") input.source
    # Container.withExec [ "rustup", "component", "add", "llvm-tools-preview" ]
    # Container.withExec [ "cargo", "install", "--locked", "cargo-llvm-cov" ]
    # Container.withExec [ "cp", "-r", "fixtures/audio", "/tmp/audio" ]
    # Container.withExec [ "cp", "fixtures/asound.conf", "/etc/asound.conf" ]
    # Container.withEnvVariable "DATABASE_URL" "sqlite:///tmp/music-player.sqlite3"
    # Container.withExec [ "cargo", "run", "--locked", "--manifest-path", "migration/Cargo.toml" ]
    # Container.withEnvVariable "MUSIC_PLAYER_APPLICATION_DIRECTORY" "/tmp"
    # Container.withEnvVariable "MUSIC_PLAYER_MUSIC_DIRECTORY" "/tmp/audio"
    # Container.withEnvVariable "MUSIC_PLAYER_DATABASE_URL" "sqlite:///tmp/music-player.sqlite3"
    # Container.withEnvVariable "MUSIC_PLAYER_PORT" "5040"
    # Container.withEnvVariable "MUSIC_PLAYER_AUDIO_OUTPUT" "fifo:/dev/null"
    # Container.withExec [ "cargo", "build", "--locked", "-p", "music-player" ]
    # Container.withExec [ "cp", "target/debug/music-player", "/usr/local/bin/music-player" ]
    # Container.withExec [ "music-player", "scan" ]
    # Base.shell """
music-player > /tmp/music-player.log 2>&1 &
daemon_pid=$!
cleanup() {
  kill -TERM "$daemon_pid" 2>/dev/null || true
  wait "$daemon_pid" 2>/dev/null || true
}
trap cleanup EXIT
# Wait for the server socket instead of assuming startup takes three seconds.
ready=false
for attempt in $(seq 1 60); do
  if ! kill -0 "$daemon_pid" 2>/dev/null; then
    cat /tmp/music-player.log >&2
    exit 1
  fi
  if (echo > /dev/tcp/127.0.0.1/5040) 2>/dev/null; then ready=true; break; fi
  sleep 1
done
if [ "$ready" != true ]; then cat /tmp/music-player.log >&2; exit 1; fi
cargo llvm-cov --locked --all-features --lib --workspace --lcov --output-path /src/lcov.info
"""
    # Container.file "/src/lcov.info"
