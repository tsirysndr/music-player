{
  lib,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  protobuf,
  makeWrapper,
  openssl,
  zstd,
  duckdb,
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
  __structuredAttrs = true;

  pname = "music-player-desktop";
  version = "0.4.4";

  src = fetchFromGitHub {
    owner = "tsirysndr";
    repo = "music-player";
    tag = "v${finalAttrs.version}";
    hash = lib.fakeHash; # fill in once the v0.4.4 tag is pushed
  };

  cargoHash = lib.fakeHash; # fill in once the v0.4.4 tag is pushed

  cargoBuildFlags = [
    "--package"
    "music-player-desktop"
  ];

  nativeBuildInputs = [
    pkg-config
    protobuf
    makeWrapper
  ];

  buildInputs = [
    duckdb
    openssl
    zstd
    alsa-lib
    freetype
  ]
  ++ runtimeLibs;

  env.ZSTD_SYS_USE_PKG_CONFIG = true;
  # DuckDB comes from nixpkgs, not from scripts/fetch-duckdb.sh: a nix build
  # has no network. These override the static defaults in .cargo/config.toml,
  # which point at a vendor/ directory that only exists after that script runs.
  env.DUCKDB_LIB_DIR = "${duckdb}/lib";
  env.DUCKDB_INCLUDE_DIR = "${duckdb}/include";
  env.DUCKDB_STATIC = "0";

  doCheck = false;

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
