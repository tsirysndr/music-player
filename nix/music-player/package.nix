{
  lib,
  stdenv,
  rustPlatform,
  fetchFromGitHub,
  pkg-config,
  protobuf,
  openssl,
  zstd,
  duckdb,
  alsa-lib,
}:

rustPlatform.buildRustPackage (finalAttrs: {
  __structuredAttrs = true;

  pname = "music-player";
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
    "music-player"
  ];

  nativeBuildInputs = [
    pkg-config
    protobuf
  ];

  buildInputs = [
    duckdb
    openssl
    zstd
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [
    alsa-lib
  ];

  env.ZSTD_SYS_USE_PKG_CONFIG = true;
  # DuckDB comes from nixpkgs, not from scripts/fetch-duckdb.sh: a nix build
  # has no network. These override the static defaults in .cargo/config.toml,
  # which point at a vendor/ directory that only exists after that script runs.
  env.DUCKDB_LIB_DIR = "${duckdb}/lib";
  env.DUCKDB_INCLUDE_DIR = "${duckdb}/include";
  env.DUCKDB_STATIC = "0";

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
