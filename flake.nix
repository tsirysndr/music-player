{
  description = "Music Player - An extensible music server written in Rust 🚀🎵✨";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;

        # The cargo sources plus everything the build scripts and
        # rust-embed pull in at compile time: the gRPC protos and the
        # committed web UI bundle.
        protoFilter = path: _type: builtins.match ".*proto$" path != null;
        webuiFilter = path: _type:
          builtins.match ".*webui/musicplayer/build.*" path != null;
        srcFilter = path: type:
          (protoFilter path type)
          || (webuiFilter path type)
          || (craneLib.filterCargoSources path type);

        src = lib.cleanSourceWith {
          src = ./.;
          filter = srcFilter;
        };

        commonArgs = {
          inherit src;
          strictDeps = true;

          nativeBuildInputs = [
            pkgs.pkg-config
            pkgs.protobuf
          ];

          buildInputs = [
            pkgs.zstd
            pkgs.openssl
          ] ++ lib.optionals pkgs.stdenv.isLinux [
            pkgs.alsa-lib
          ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        music-player = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          # The test-suite needs audio fixtures and a running server;
          # it is exercised by the regular CI, not by the nix build.
          doCheck = false;
        });
      in
      {
        checks = {
          inherit music-player;

          music-player-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets";
          });

          music-player-fmt = craneLib.cargoFmt {
            inherit src;
          };
        };

        packages.default = music-player;

        apps.default = flake-utils.lib.mkApp {
          drv = music-player;
        };

        devShells.default = craneLib.devShell {
          inputsFrom = [ music-player ];

          packages = with pkgs; [
            bun
            protobuf
            sqlite
          ];
        };
      });
}
