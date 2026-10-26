module CI.Nix where

import Prelude
import CI.Base as Base
import Dagger (Client)
import Dagger.Container as Container
import Dagger.Types (Container, Directory, Platform)
import Effect.Aff (Aff, throwError)
import Effect.Exception (error)

prepared :: Client -> Platform -> Directory -> Container
prepared client arch source = Base.container client arch "nixos/nix:2.31.2"
  # Container.withEnvVariable "NIX_CONFIG" "experimental-features = nix-command flakes\nfilter-syscalls = false\nsandbox = false\n"
  # Container.withDirectory "/src" (Base.sourceTree client source)
  # Container.withWorkdir "/src"

flake :: Client -> { source :: Directory, platform :: String } -> Aff String
flake client input = do
  arch <- Base.platform input.platform
  prepared client arch input.source
    # Container.withExec [ "nix", "build", "-L", "path:." ]
    # Container.withExec [ "sh", "-ec", "test -x result/bin/music-player; ls -la result/bin/" ]
    # Container.stdout

package :: Client -> { source :: Directory, platform :: String, attribute :: String } -> Aff Directory
package client input = do
  case input.attribute of
    "music-player" -> pure unit
    "music-player-desktop" -> pure unit
    _ -> throwError (error "Unknown Nix package attribute")
  arch <- Base.platform input.platform
  pure $ prepared client arch input.source
    # Container.withExec [ "nix-build", "nix", "-A", input.attribute, "-I", "nixpkgs=channel:nixos-unstable" ]
    # Container.withExec [ "sh", "-ec", "mkdir -p /out; readlink -f result > /out/store-path; nix copy --to file:///out/store $(cat /out/store-path); ls -la result/bin/" ]
    # Container.directory "/out"
