module CI.Release where

import Prelude
import CI.Base as Base
import CI.Rust as Rust
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Types (Directory)
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)

type Input = { source :: Directory, target :: String, version :: String, desktop :: Boolean }

linux :: Client -> Input -> Aff Directory
linux client input = do
  settings <- case input.target of
    "x86_64-unknown-linux-gnu" -> pure { platform: "linux/amd64", arch: "x86_64" }
    "aarch64-unknown-linux-gnu" -> pure { platform: "linux/arm64", arch: "aarch64" }
    _ -> throwError (error "Linux releases support x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu")
  arch <- Base.platform settings.platform
  let
    packages = if input.desktop then [ "-p", "music-player", "-p", "music-player-desktop" ] else []
    binary = if input.desktop then "music-player-desktop" else "music-player"
    built = Rust.prepared client arch (input.target <> "-release") input.source
      # Container.withExec [ "rustup", "target", "add", input.target ]
      # Container.withExec ([ "cargo", "build", "--locked", "--release", "--target", input.target ] <> packages)
      # Container.withEnvVariable "RELEASE_VERSION" input.version
      # Container.withEnvVariable "RELEASE_TARGET" input.target
      # Container.withEnvVariable "RELEASE_BINARY" binary
      # Base.shell """
[[ "$RELEASE_VERSION" == latest || "$RELEASE_VERSION" =~ ^v?[0-9][0-9A-Za-z.+~-]*$ ]] || { echo 'Invalid release version' >&2; exit 2; }
mkdir -p /out
asset="${RELEASE_BINARY}_${RELEASE_VERSION}_${RELEASE_TARGET}.tar.gz"
tar czf "/out/$asset" -C "target/$RELEASE_TARGET/release" "$RELEASE_BINARY"
cd /out
sha256sum "$asset" > "$asset.sha256"
"""
    packaged = if input.desktop then built
      # Container.withExec [ "bash", "dist/package-linux.sh", "all", input.version, settings.arch, "target/" <> input.target <> "/release" ]
      # Base.shell "mkdir -p /out/dist; cp dist/*.deb dist/*.deb.sha256 dist/*.rpm dist/*.rpm.sha256 /out/dist/"
      else built
  pure (Container.directory "/out" packaged)
