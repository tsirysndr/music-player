module CI.Rust where

import Prelude
import CI.Base as Base
import CI.Web as Web
import Data.Maybe (Maybe(..))
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Query as Query
import Dagger.Types (CacheSharingMode(..), Container, Directory, File, Platform)
import Effect.Aff (Aff)

prepared :: Client -> Platform -> String -> Directory -> Container
prepared client arch purpose source = Base.container client arch "rust:1.98.0-bookworm"
  # Base.shell "apt-get update && apt-get install -y --no-install-recommends libasound2-dev pkg-config protobuf-compiler libfontconfig1-dev libdbus-1-dev libxkbcommon-dev libwayland-dev curl unzip ca-certificates && rm -rf /var/lib/apt/lists/*"
  # Container.withMountedCache "/usr/local/cargo/registry" (Query.cacheVolume "music-player-cargo-registry" client)
  # Container.withMountedCache "/usr/local/cargo/git" (Query.cacheVolume "music-player-cargo-git" client)
  # Container.withDirectory "/src" (Base.sourceTree client source)
  # Container.withDirectory "/src/webui/musicplayer/build" (Web.build client arch source)
  # Container.withWorkdir "/src"
  # Container.withMountedCacheWith "/src/target" (Query.cacheVolume ("music-player-rust-1.98.0-" <> purpose) client)
      (Container.defaultWithMountedCacheOpts { sharing = Just CacheSharingModeLocked })
  # Container.withExec [ "bash", "scripts/fetch-duckdb.sh" ]

type Input = { source :: Directory, platform :: String }

coverage :: Client -> Input -> Aff File
coverage client input = do
  arch <- Base.platform input.platform
  pure $ prepared client arch (input.platform <> "-coverage") input.source
    # Container.withExec [ "rustup", "component", "add", "llvm-tools-preview" ]
    # Container.withExec [ "cargo", "install", "--locked", "cargo-llvm-cov" ]
    # Container.withExec [ "cp", "-r", "fixtures/audio", "/tmp/audio" ]
    # Container.withEnvVariable "MUSIC_PLAYER_AUDIO_OUTPUT" "fifo:/dev/null"
    # Container.withExec [ "cargo", "llvm-cov", "--locked", "--workspace", "--lcov", "--output-path", "/src/lcov.info" ]
    # Container.file "/src/lcov.info"

clippy :: Client -> Input -> Aff File
clippy client input = do
  arch <- Base.platform input.platform
  pure $ prepared client arch (input.platform <> "-clippy") input.source
    # Container.withExec [ "rustup", "component", "add", "clippy" ]
    # Container.withExec [ "cargo", "install", "--locked", "clippy-sarif", "sarif-fmt" ]
    -- Findings are advisory, as in the original workflow. Conversion errors
    -- still fail, and the report is exported for GitHub's SARIF upload action.
    # Base.shell "cargo clippy --locked --all-features --message-format=json > /tmp/clippy.json || true; clippy-sarif < /tmp/clippy.json > /src/rust-clippy-results.sarif; sarif-fmt < /src/rust-clippy-results.sarif"
    # Container.file "/src/rust-clippy-results.sarif"
