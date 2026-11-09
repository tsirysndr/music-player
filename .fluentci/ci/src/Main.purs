module Main (dependencies, webBuild, webCheck, coverage, clippy, release, integration, nixBuild, nixPackage, check, codecov, gemfury, cachix, githubAssets, gemfuryDirectory, releaseVersion) where

import Prelude
import CI.Support (run, runIn, unwrap, absolute, os, isMac, files, checksum, validVersion, env)
import CI.Support as Support
import CI.Package as Package
import CI.DuckDB as DuckDB
import CI.Gemfury as Gemfury
import Data.Array as A
import Data.String as S
import Effect (Effect)
import Effect.Exception (throw)
import FluentCI.Plugin as P
import FluentCI.Host.Plan as H

-- | Install native libraries required to build and test the player.
dependencies :: Effect Unit
dependencies = do
  host <- os
  if isMac host then run [ "brew", "install", "protobuf" ]
  else if host == "linux" then runIn "."
    [ [ "sudo", "apt-get", "update" ]
    , [ "sudo", "apt-get", "install", "-y", "libasound2-dev", "pkg-config", "protobuf-compiler", "libfontconfig1-dev", "libdbus-1-dev", "libxkbcommon-dev", "libwayland-dev", "curl", "unzip", "rpm" ] ]
  else throw "Unsupported build host"

-- | Install locked frontend dependencies and build the embedded web UI.
-- | @param source Source checkout to build.
webBuild :: { source :: String } -> Effect Unit
webBuild args = runIn (args.source <> "/webui/musicplayer")
  [ [ "bun", "install", "--frozen-lockfile" ], [ "bun", "run", "build" ] ]

-- | Typecheck, test with coverage, and build the web UI and Storybook.
webCheck :: Effect Unit
webCheck = do
  runIn "webui/musicplayer"
    [ [ "bun", "install", "--frozen-lockfile" ], [ "bunx", "tsc", "--noEmit" ]
    , [ "bun", "run", "test:coverage" ], [ "bunx", "vite", "build" ], [ "bunx", "storybook", "build" ] ]
  runIn "."
    [ [ "mkdir", "-p", "artifacts/webui" ]
    , [ "cp", "-R", "webui/musicplayer/coverage", "webui/musicplayer/build", "webui/musicplayer/storybook-static", "artifacts/webui/" ] ]

-- | Run workspace tests and export LCOV coverage.
coverage :: Effect Unit
coverage = do
  webBuild { source: "." }
  target <- DuckDB.nativeTarget
  DuckDB.fetch "." target
  runIn "."
    [ [ "cargo", "install", "--locked", "cargo-llvm-cov" ]
    , [ "cp", "-R", "fixtures/audio", "/tmp/audio" ], [ "mkdir", "-p", "artifacts" ]
    , [ "env", "MUSIC_PLAYER_AUDIO_OUTPUT=fifo:/dev/null", "cargo", "llvm-cov", "--locked", "--workspace", "--lcov", "--output-path", "artifacts/lcov.info" ] ]

-- | Export advisory Clippy findings as SARIF; report conversion errors fail.
clippy :: Effect Unit
clippy = do
  webBuild { source: "." }
  target <- DuckDB.nativeTarget
  DuckDB.fetch "." target
  runIn "." [ [ "cargo", "install", "--locked", "clippy-sarif", "sarif-fmt" ], [ "mkdir", "-p", "artifacts" ] ]
  report <- Support.taskResult (H.task "clippy-report" "cargo" "."
    # H.withArgs [ "clippy", "--locked", "--all-features", "--message-format=json" ]
    # H.allowFailure true)
  run [ "cp", report.stdout, "artifacts/clippy.json" ]
  run [ "clippy-sarif", "--input", "artifacts/clippy.json", "--output", "artifacts/rust-clippy-results.sarif" ]

-- | Build and package a Linux or macOS release, including desktop installers.
-- | @param source Source checkout; tooling stays at the workflow revision.
-- | @param target Rust target triple.
-- | @param version Release tag used in filenames and package metadata.
-- | @param desktop Build the Slint desktop client and native installers.
release :: { source :: String, target :: String, version :: String, desktop :: Boolean } -> Effect Unit
release args = do
  unless (validVersion args.version || (args.version == "latest" && not args.desktop)) (throw "Invalid release version")
  host <- os
  arch <- P.getArch >>= unwrap
  let mac = isMac host
      supported = if mac then A.elem args.target [ "x86_64-apple-darwin", "aarch64-apple-darwin" ]
        else host == "linux" && ((arch == "x86_64" && args.target == "x86_64-unknown-linux-gnu") || (A.elem arch [ "aarch64", "arm64" ] && args.target == "aarch64-unknown-linux-gnu"))
  unless supported (throw "Target does not match the release runner")
  source <- absolute args.source
  tooling <- absolute "."
  let out = tooling <> "/artifacts/release"
      binary = if args.desktop then "music-player-desktop" else "music-player"
      binaryDir = source <> "/target/" <> args.target <> "/release"
      asset = binary <> "_" <> args.version <> "_" <> args.target <> ".tar.gz"
      packages = if args.desktop then [ "-p", "music-player", "-p", "music-player-desktop" ] else []
  run [ "mkdir", "-p", out ]
  DuckDB.fetch source args.target
  webBuild { source }
  runIn source [ [ "cargo", "build", "--locked", "--release", "--target", args.target ] <> packages ]
  if args.desktop && mac then Package.mac binaryDir args.version args.target asset out
  else do
    run [ "tar", "czf", out <> "/" <> asset, "-C", binaryDir, binary ]
    when args.desktop do
      let packageArch = if args.target == "x86_64-unknown-linux-gnu" then "x86_64" else "aarch64"
      Package.linux binaryDir args.version packageArch out
  checksum (out <> "/" <> asset)

packageFile :: String -> Boolean
packageFile path = S.endsWith (S.Pattern ".deb") path || S.endsWith (S.Pattern ".rpm") path

-- | Test against a managed daemon, then package the native Linux release.
-- | @param target Native Linux Rust target triple.
integration :: { target :: String } -> Effect Unit
integration args = do
  webBuild { source: "." }
  DuckDB.fetch "." args.target
  runIn "." [ [ "cargo", "install", "--locked", "cargo-llvm-cov" ], [ "mkdir", "-p", "artifacts" ], [ "cp", "-R", "fixtures/audio", "/tmp/audio" ], [ "sudo", "cp", "fixtures/asound.conf", "/etc/asound.conf" ], [ "env", "DATABASE_URL=sqlite:///tmp/music-player.sqlite3", "cargo", "run", "--locked", "--manifest-path", "migration/Cargo.toml" ], [ "cargo", "build", "--locked", "-p", "music-player" ] ]
  workspace <- absolute "."
  path <- env "PATH" ""
  let configured = P.pipeline "player"
        # P.withEnv "PATH" (workspace <> "/target/debug:" <> path)
        # P.withEnv "DATABASE_URL" "sqlite:///tmp/music-player.sqlite3"
        # P.withEnv "MUSIC_PLAYER_APPLICATION_DIRECTORY" "/tmp"
        # P.withEnv "MUSIC_PLAYER_MUSIC_DIRECTORY" "/tmp/audio"
        # P.withEnv "MUSIC_PLAYER_DATABASE_URL" "sqlite:///tmp/music-player.sqlite3"
        # P.withEnv "MUSIC_PLAYER_PORT" "5040"
        # P.withEnv "MUSIC_PLAYER_AUDIO_OUTPUT" "fifo:/dev/null"
  void (P.stdout (configured # P.exec [ "target/debug/music-player", "scan" ]) >>= unwrap)
  let daemon = configured # P.exec [ "target/debug/music-player" ] # P.asService "music-player"
  void (P.stdout (configured # P.withService daemon # P.waitOn 5040 60
    # P.exec [ "cargo", "llvm-cov", "--locked", "--all-features", "--lib", "--workspace", "--lcov", "--output-path", "artifacts/lcov.info" ]) >>= unwrap)
  release { source: ".", target: args.target, version: "latest", desktop: false }

-- | Build the local flake on the native runner.
nixBuild :: Effect Unit
nixBuild = Support.nixBuild { source: ".", cache: "" }

-- | Build a nixpkgs-ready package from the released source.
-- | @param attribute music-player or music-player-desktop.
nixPackage :: { attribute :: String } -> Effect Unit
nixPackage args = do
  unless (A.elem args.attribute [ "music-player", "music-player-desktop" ]) (throw "Unknown Nix package attribute")
  run [ "nix-build", "nix", "-A", args.attribute, "-I", "nixpkgs=channel:nixos-unstable" ]
  run [ "test", "-x", "result/bin/" <> args.attribute ]

-- | Run module regression tests and lint every GitHub workflow.
check :: Effect Unit
check = do
  run [ "node", "--test", ".fluentci/tests/ci.test.mjs" ]
  paths <- files ".github/workflows"
  run ([ "go", "run", "github.com/rhysd/actionlint/cmd/actionlint@v1.7.12", "-shellcheck=" ] <> A.filter (S.endsWith (S.Pattern ".yml")) paths)

-- | Upload a coverage report using the CLI installed by setup-codecov.
-- | @param report LCOV report path.
-- | @param commit Commit that generated the report.
-- | @param flag Coverage flag; use an empty string for workspace coverage.
codecov :: { report :: String, commit :: String, flag :: String } -> Effect Unit
codecov args = run ([ "codecovcli", "upload-process", "--fail-on-error", "--disable-search", "-f", args.report, "-C", args.commit ] <> if args.flag == "" then [] else [ "-F", args.flag ])

-- | Upload explicit package files with host credentials.
-- | @param files Debian and RPM package paths.
gemfury :: { files :: Array String } -> Effect Unit
gemfury args = Gemfury.uploadFiles args.files

-- | Select Debian/RPM packages and upload using host credentials.
-- | @param directory Artifact directory to scan recursively.
gemfuryDirectory :: { directory :: String } -> Effect Unit
gemfuryDirectory args = do
  token <- env "FURY_TOKEN" ""
  when (S.trim token /= "") do
    paths <- A.filter packageFile <$> files args.directory
    Gemfury.uploadFiles paths

-- | Configure or push a Cachix cache using host credentials.
-- | @param operation setup or push.
-- | @param cache Cachix cache name.
cachix :: { operation :: String, cache :: String } -> Effect Unit
cachix = Support.cachix

-- | Upload release files to an existing GitHub release.
-- | @param directory Release artifact directory.
-- | @param tag Release tag.
githubAssets :: { directory :: String, tag :: String } -> Effect Unit
githubAssets args = Support.githubAssets { directory: args.directory, tag: args.tag, create: false }

-- | Select and persist the release version using the workflow input or GitHub ref.
-- | @param tag Optional workflow input; empty selects GITHUB_REF_NAME.
releaseVersion :: { tag :: String } -> Effect Unit
releaseVersion args = do
  version <- if args.tag == "" then Support.requiredEnv "GITHUB_REF_NAME" else pure args.tag
  unless (validVersion version || version == "latest") (throw "Invalid release version")
  destination <- Support.requiredEnv "GITHUB_ENV"
  previous <- Support.readText destination
  Support.writeText destination (previous <> "RELEASE_VERSION=" <> version <> "\n")
