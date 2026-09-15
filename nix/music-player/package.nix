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
  __structuredAttrs = true;

  pname = "music-player";
  version = "0.4.3";

  src = fetchFromGitHub {
    owner = "tsirysndr";
    repo = "music-player";
    tag = "v${finalAttrs.version}";
    hash = lib.fakeHash; # fill in once the v0.4.3 tag is pushed
  };

  cargoHash = lib.fakeHash; # fill in once the v0.4.3 tag is pushed

  cargoBuildFlags = [
    "--package"
    "music-player"
  ];

  nativeBuildInputs = [
    pkg-config
    protobuf
  ];

  buildInputs = [
    openssl
    zstd
  ]
  ++ lib.optionals stdenv.hostPlatform.isLinux [
    alsa-lib
  ];

  env.ZSTD_SYS_USE_PKG_CONFIG = true;

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
