# Nixpkgs package for the Slint desktop client; the nixpkgs copy goes to
# pkgs/by-name/mu/music-player-desktop/package.nix. Linux only — on macOS
# the .app bundle is produced by dist/package-macos.sh, not nix.
{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  protobuf,
  makeWrapper,
  openssl,
  zstd,
  alsa-lib,
  freetype,
  fontconfig,
  wayland,
  libxkbcommon,
  libGL,
  libx11,
  libxcursor,
  libxi,
  libxrandr,
}:

let
  # winit and Slint's GL renderer open these at *runtime* (dlopen, not
  # linked), so they must be on the wrapped binary's search path — a nix
  # build has no /usr/lib to fall back on.
  runtimeLibs = [
    wayland
    libxkbcommon
    libGL
    fontconfig
    libx11
    libxcursor
    libxi
    libxrandr
  ];
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = "music-player-desktop";
  version = "0.4.2";

  src = fetchFromGitHub {
    owner = "tsirysndr";
    repo = "music-player";
    tag = "v${finalAttrs.version}";
    hash = "sha256-ZcLDw9+bH3Iu3zdIyFdlIeAQfsnySDnfeJoHr2yViuA=";
  };

  # Same workspace lockfile as music-player, so the vendor hash is shared —
  # copy the value from music-player.nix once it is filled in.
  cargoHash = lib.fakeHash;

  cargoBuildFlags = [ "--package" "music-player-desktop" ];

  nativeBuildInputs = [
    pkg-config
    protobuf # gRPC codegen (tonic-build)
    makeWrapper
  ];

  buildInputs = [
    openssl
    zstd
    alsa-lib
    freetype
  ] ++ runtimeLibs;

  env.ZSTD_SYS_USE_PKG_CONFIG = true;

  # The test-suite needs audio fixtures and a running server; it is
  # exercised by the project's CI, not by the nix build.
  doCheck = false;

  # The same share/ tree the deb and rpm install, so the app shows up in
  # launchers with its icon.
  postInstall = ''
    install -Dm644 dist/music-player.desktop \
      $out/share/applications/music-player.desktop
    install -Dm644 desktop/assets/icon.svg \
      $out/share/icons/hicolor/scalable/apps/music-player.svg
  '';

  postFixup = ''
    wrapProgram $out/bin/music-player-desktop \
      --prefix LD_LIBRARY_PATH : ${lib.makeLibraryPath runtimeLibs}
  '';

  meta = {
    description = "Desktop client (Slint) for the music-player daemon";
    homepage = "https://github.com/tsirysndr/music-player";
    changelog = "https://github.com/tsirysndr/music-player/blob/v${finalAttrs.version}/CHANGELOG.md";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ tsirysndr ];
    mainProgram = "music-player-desktop";
    platforms = lib.platforms.linux;
  };
})
