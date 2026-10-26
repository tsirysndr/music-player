module Main where

import Prelude
import CI.Integration as Integration
import CI.Pipeline as Pipeline
import CI.Nix as Nix
import CI.Release as Release
import CI.Rust as Rust
import CI.Web as Web
import Dagger.Modules as Module
import Dagger.Types (Directory)
import Effect (Effect)

sourceAndPlatform :: Module.Arguments { source :: Directory, platform :: String }
sourceAndPlatform = ado
  source <- Module.argument "source"
  platform <- Module.defaultArgument "platform" "linux/amd64"
  in { source, platform }

main :: Effect Unit
main = Module.runModule
  [ Module.object_ "MusicPlayerCi" {}
      [ Module.function "test" "Migrate the database, scan fixtures, and test against a running daemon with coverage."
          sourceAndPlatform Integration.test
      , Module.function "build" "Build and package the Linux daemon; replaces the standalone build job."
          (ado
            source <- Module.argument "source"
            target <- Module.defaultArgument "target" "x86_64-unknown-linux-gnu"
            tag <- Module.defaultArgument "tag" "latest"
            in { source, target, tag }) Pipeline.build
      , Module.function "pipeline" "Run integration tests, then build release artifacts."
          (ado
            source <- Module.argument "source"
            platform <- Module.defaultArgument "platform" "linux/amd64"
            target <- Module.defaultArgument "target" "x86_64-unknown-linux-gnu"
            tag <- Module.defaultArgument "tag" "latest"
            in { source, platform, target, tag }) Pipeline.run
      , Module.function "webCheck" "Typecheck, test, build the web UI and Storybook; return artifacts."
          sourceAndPlatform Web.check
      , Module.function "rustCoverage" "Run workspace tests with LLVM coverage and audio fixtures."
          sourceAndPlatform Rust.coverage
      , Module.function "clippy" "Run advisory Clippy analysis and return a SARIF report."
          sourceAndPlatform Rust.clippy
      , Module.function "linuxRelease" "Build Linux binaries and optionally desktop deb/rpm packages."
          (ado
            source <- Module.argument "source"
            target <- Module.argument "target"
            version <- Module.argument "version"
            desktop <- Module.defaultArgument "desktop" false
            in { source, target, version, desktop }) Release.linux
      , Module.function "nixFlake" "Build the flake on Linux."
          sourceAndPlatform Nix.flake
      , Module.function "nixPackage" "Build a nixpkgs-ready Linux package."
          (ado
            source <- Module.argument "source"
            platform <- Module.defaultArgument "platform" "linux/amd64"
            attribute <- Module.defaultArgument "attribute" "music-player"
            in { source, platform, attribute }) Nix.package
      ]
  ]
