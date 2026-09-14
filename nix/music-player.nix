# Nixpkgs package for the music-player daemon/CLI, kept in the shape
# nixpkgs expects: copy this file to pkgs/by-name/mu/music-player/package.nix
# in a nixpkgs checkout to open the PR. Unlike ../flake.nix it builds from
# the released tag, not the working tree.
{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  protobuf,
  openssl,
  zstd,
  alsa-lib,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "music-player";
  version = "0.4.2";

  src = fetchFromGitHub {
    owner = "tsirysndr";
    repo = "music-player";
    tag = "v${finalAttrs.version}";
    hash = "sha256-ZcLDw9+bH3Iu3zdIyFdlIeAQfsnySDnfeJoHr2yViuA=";
  };

  # TODO before the nixpkgs PR: build once — the failure prints the real
  # hash on the "got:" line. Same value goes in music-player-desktop.nix
  # (both packages vendor the same workspace Cargo.lock).
  cargoHash = lib.fakeHash;

  # Only the daemon/CLI: the Slint desktop client is a workspace member but
  # drags in the whole GUI stack (fontconfig, wayland, xkbcommon, libGL).
  # It is packaged separately as music-player-desktop.
  cargoBuildFlags = [ "--package" "music-player" ];

  nativeBuildInputs = [
    pkg-config
    protobuf # gRPC codegen (tonic-build)
  ];

  buildInputs = [
    openssl
    zstd
  ] ++ lib.optionals stdenv.hostPlatform.isLinux [
    alsa-lib
  ];

  env.ZSTD_SYS_USE_PKG_CONFIG = true;

  # The test-suite needs audio fixtures and a running server; it is
  # exercised by the project's CI, not by the nix build.
  doCheck = false;

  meta = {
    description = "Extensible music player daemon written in Rust";
    homepage = "https://github.com/tsirysndr/music-player";
    changelog = "https://github.com/tsirysndr/music-player/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ tsirysndr ];
    mainProgram = "music-player";
    platforms = lib.platforms.linux ++ lib.platforms.darwin;
  };
})
