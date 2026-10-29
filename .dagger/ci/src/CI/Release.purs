module CI.Release where

import Prelude
import CI.Base as Base
import CI.Rust as Rust
import Data.Array as Array
import Data.Foldable (all, foldM)
import Data.String as S
import Data.String.CodeUnits as CU
import Dagger (Client)
import Dagger.Container as C
import Dagger.Directory as D
import Dagger.File as F
import Dagger.Query as Q
import Dagger.Types (Container, Directory, File)
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)
import Release.Common as Common
import Release.Deb as Deb
import Release.Rpm as Rpm
import Release.Sha256 as Sha256
import Release.TarGz as TarGz

type Input = { source :: Directory, target :: String, version :: String, desktop :: Boolean }

metadataVersion :: String -> String
metadataVersion value = if S.take 1 value == "v" then S.drop 1 value else value

validVersion :: String -> Boolean
validVersion value = value == "latest" ||
  let version = metadataVersion value
  in Array.elem (S.take 1 version) [ "0", "1", "2", "3", "4", "5", "6", "7", "8", "9" ]
    && all (\c -> (c >= '0' && c <= '9') || (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || Array.elem c [ '.', '+', '~', '-' ]) (CU.toCharArray version)

withChecksum :: Client -> String -> File -> Directory -> Aff Directory
withChecksum client name file output = do
  checksum <- Sha256.generate client { file, name: Rpm.basename name }
  pure $ output # D.withFile name file # D.withFile (name <> ".sha256") checksum

-- Project-specific staging; mechanics are shared with linux-packaging.
stageDesktop :: String -> Container -> Container
stageDesktop binaryDir = C.withExec [ "install", "-d", "/stage/usr/bin", "/stage/usr/share/applications", "/stage/usr/share/icons/hicolor/scalable/apps", "/stage/usr/share/licenses/music-player" ]
  >>> C.withExec [ "install", "-m", "0755", binaryDir <> "/music-player", binaryDir <> "/music-player-desktop", "/stage/usr/bin/" ]
  >>> C.withExec [ "install", "-m", "0644", "dist/music-player.desktop", "/stage/usr/share/applications/music-player.desktop" ]
  >>> C.withExec [ "install", "-m", "0644", "desktop/assets/icon.svg", "/stage/usr/share/icons/hicolor/scalable/apps/music-player.svg" ]
  >>> C.withExec [ "install", "-m", "0644", "LICENSE", "/stage/usr/share/licenses/music-player/LICENSE" ]

replace :: String -> String -> String -> String
replace needle value = S.replaceAll (S.Pattern needle) (S.Replacement value)

stampControl :: String -> String -> String -> String
stampControl version arch = replace "@VERSION@" version >>> replace "@DEB_ARCH@" arch

stampSpec :: String -> String -> String -> String
stampSpec version arch = replace "@VERSION@" (replace "-" "~" version) >>> replace "@RPM_ARCH@" arch

packageDesktop :: Client -> String -> String -> String -> Directory -> Container -> Directory -> Aff Directory
packageDesktop client version arch binaryDir source built output = do
  let staged = stageDesktop binaryDir built
      root = C.directory "/stage" staged
      debArch = if arch == "x86_64" then "amd64" else "arm64"
      debName = "music-player_" <> version <> "_" <> debArch <> ".deb"
  control <- F.contents (D.file "dist/debian-control.in" source)
  spec <- F.contents (D.file "dist/music-player.spec.in" source)
  deb <- Deb.build client
    { root: D.withNewFile "DEBIAN/control" (stampControl version debArch control) root
    , name: debName
    }
  withDeb <- withChecksum client ("dist/" <> debName) deb output
  rpms <- Rpm.build client
    { sources: Q.directory client # D.withDirectory "root" root
    , spec: Q.directory client # D.withNewFile "package.spec" (stampSpec version arch spec) # D.file "package.spec"
    , target: arch
    }
  paths <- D.glob "**/*.rpm" rpms
  foldM (\out path -> withChecksum client ("dist/" <> Rpm.basename path) (D.file path rpms) out) withDeb paths

linux :: Client -> Input -> Aff Directory
linux client input = do
  Common.ensure (validVersion input.version) "Invalid release version"
  Common.ensure (not input.desktop || input.version /= "latest") "Desktop packages require a numeric version"
  settings <- case input.target of
    "x86_64-unknown-linux-gnu" -> pure { platform: "linux/amd64", arch: "x86_64" }
    "aarch64-unknown-linux-gnu" -> pure { platform: "linux/arm64", arch: "aarch64" }
    _ -> throwError (error "Linux releases support x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu")
  arch <- Base.platform settings.platform
  let packages = if input.desktop then [ "-p", "music-player", "-p", "music-player-desktop" ] else []
      binary = if input.desktop then "music-player-desktop" else "music-player"
      binaryDir = "target/" <> input.target <> "/release"
      asset = binary <> "_" <> input.version <> "_" <> input.target <> ".tar.gz"
      built = Rust.prepared client arch (input.target <> "-release") input.source
        # C.withExec [ "rustup", "target", "add", input.target ]
        # C.withExec ([ "cargo", "build", "--locked", "--release", "--target", input.target ] <> packages)
      -- Snapshot only the published binary out of the mutable target cache.
      stagedArchive = built # C.withExec [ "install", "-D", "-m", "0755", binaryDir <> "/" <> binary, "/archive/" <> binary ]
  archive <- TarGz.create client { source: C.directory "/archive" stagedArchive, name: asset, members: [ binary ] }
  output <- withChecksum client asset archive (Q.directory client)
  if input.desktop then packageDesktop client (metadataVersion input.version) settings.arch binaryDir input.source built output
  else pure output
