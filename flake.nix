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
        # (The desktop client's ui/ and assets/ trees are left out of the
        # CLI build — the desktop package widens this filter below.)
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

        # What the desktop additionally compiles in — the Slint UI, its
        # fonts and icons, the builtin skins (include_str!) — plus what its
        # install step ships: the desktop entry and the license, mirroring
        # dist/package-linux.sh.
        desktopAssetFilter = path: _type:
          builtins.match ".*desktop/(ui|assets|skins).*" path != null
          || lib.hasSuffix "dist/music-player.desktop" path
          || lib.hasSuffix "LICENSE" path;

        desktopSrc = lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            (desktopAssetFilter path type) || (srcFilter path type);
        };

        commonArgs = {
          inherit src;
          pname = "music-player";
          version = "0.4.2";
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
          ] ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
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

        # ── Desktop client (Linux only) ─────────────────────────────────
        # macOS gets the CLI alone from this flake: there is no desktop-
        # entry convention there, and the .app bundle is the job of
        # dist/package-macos.sh. Evaluation is lazy, so none of this is
        # forced on darwin — the package set below simply leaves it out.

        # The libraries winit and Slint's GL renderer open at *runtime*
        # (dlopen, not linked), so they must be on the wrapped binary's
        # search path — a nix build has no /usr/lib to fall back on.
        desktopRuntimeLibs = [
          pkgs.wayland
          pkgs.libxkbcommon
          pkgs.libGL
          pkgs.fontconfig
          pkgs.libx11
          pkgs.libxcursor
          pkgs.libxi
          pkgs.libxrandr
        ];

        desktopArgs = commonArgs // {
          src = desktopSrc;
          pname = "music-player-desktop";
          cargoExtraArgs = "--locked --package music-player-desktop";
          nativeBuildInputs = commonArgs.nativeBuildInputs ++ [
            pkgs.makeWrapper
          ];
          buildInputs = commonArgs.buildInputs
            ++ [ pkgs.freetype ]
            ++ desktopRuntimeLibs;
        };

        # Its own dependency build: the GUI stack (slint, winit, GL) is not
        # in the CLI's tree, and sharing artifacts would rebuild both ways.
        desktopCargoArtifacts = craneLib.buildDepsOnly desktopArgs;

        music-player-desktop = craneLib.buildPackage (desktopArgs // {
          cargoArtifacts = desktopCargoArtifacts;
          doCheck = false;

          # The same share/ tree the deb and rpm install, so the app shows
          # up in launchers with its icon.
          postInstall = ''
            install -Dm644 dist/music-player.desktop \
              $out/share/applications/music-player.desktop
            install -Dm644 desktop/assets/icon.svg \
              $out/share/icons/hicolor/scalable/apps/music-player.svg
            install -Dm644 LICENSE \
              $out/share/licenses/music-player/LICENSE
          '';

          postFixup = ''
            wrapProgram $out/bin/music-player-desktop \
              --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath desktopRuntimeLibs}
          '';
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

        # Both binaries on Linux; the CLI alone on darwin.
        packages = {
          default = music-player;
          inherit music-player;
        } // lib.optionalAttrs pkgs.stdenv.hostPlatform.isLinux {
          inherit music-player-desktop;
        };

        apps.default = flake-utils.lib.mkApp {
          drv = music-player;
        };

        devShells.default = craneLib.devShell {
          # Every workspace member's build environment, so a plain
          # `cargo build -p <anything>` works from this shell — on Linux
          # that includes the desktop's GUI stack (fontconfig, wayland,
          # xkbcommon, GL, X11), which the CLI alone would not pull in.
          inputsFrom = [ music-player ]
            ++ lib.optionals pkgs.stdenv.hostPlatform.isLinux [
              music-player-desktop
            ];

          packages = with pkgs; [
            bun # webui/musicplayer build
            deno # extension/tooling scripts
            mise # per-project tool versions and tasks
            nodejs # npm package + anything bun does not cover
            protobuf # gRPC codegen
            sqlite # poke the library database directly
          ];

          # The desktop dlopens these at runtime rather than linking them,
          # so `cargo run -p music-player-desktop` from this shell needs
          # them findable — same set the installed binary is wrapped with.
          LD_LIBRARY_PATH = lib.optionalString pkgs.stdenv.hostPlatform.isLinux (
            lib.makeLibraryPath desktopRuntimeLibs
          );
        };
      });
}
