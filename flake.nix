{
  description = "Music Player - An extensible music server written in Rust 🚀🎵✨";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    crane.url = "github:ipetkov/crane";

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachSystem [
      "x86_64-linux"
      "aarch64-linux"
      "aarch64-darwin"
    ] (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };

        inherit (pkgs) lib;

        craneLib = crane.mkLib pkgs;

        # The cargo sources plus everything the build scripts, `include_str!`
        # and rust-embed pull in at compile time: the gRPC protos, the
        # committed web UI bundle and the extension manifest schema.
        # (The desktop client's ui/ and assets/ trees are left out — it is
        # not part of this build, see cargoExtraArgs below.)
        protoFilter = path: _type: builtins.match ".*proto$" path != null;
        webuiFilter = path: _type:
          builtins.match ".*webui/musicplayer/build.*" path != null;
        schemaFilter = path: _type:
          lib.hasSuffix "extensions/schema.yaml" path;
        srcFilter = path: type:
          (protoFilter path type)
          || (webuiFilter path type)
          || (schemaFilter path type)
          || (craneLib.filterCargoSources path type);

        src = lib.cleanSourceWith {
          src = ./.;
          filter = srcFilter;
        };

        commonArgs = {
          inherit src;
          pname = "music-player";
          version = "0.3.0";
          strictDeps = true;

          # Only the daemon/CLI: the Slint desktop client is a workspace
          # member but drags in the whole GUI stack (fontconfig, wayland,
          # xkbcommon, libGL), which this build has no business pulling in.
          cargoExtraArgs = "--locked --package music-player";

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
