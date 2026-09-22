fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Without this Cargo caches the build script's output and a `.proto` edit
    // never regenerates `src/api` — the change compiles against last build's
    // bindings and fails with "no such message", pointing nowhere useful.
    println!("cargo:rerun-if-changed=proto");

    tonic_prost_build::configure()
        .out_dir("src/api")
        .compile_protos(
            &[
                "proto/metadata/v1alpha1/artist.proto",
                "proto/metadata/v1alpha1/album.proto",
                "proto/metadata/v1alpha1/lyrics.proto",
                "proto/metadata/v1alpha1/track.proto",
                "proto/objects/v1alpha1/addon.proto",
                "proto/objects/v1alpha1/playlist.proto",
                "proto/objects/v1alpha1/tracklist.proto",
                "proto/music/v1alpha1/addons.proto",
                "proto/music/v1alpha1/analysis.proto",
                "proto/music/v1alpha1/analytics.proto",
                "proto/music/v1alpha1/core.proto",
                "proto/music/v1alpha1/history.proto",
                "proto/music/v1alpha1/library.proto",
                "proto/music/v1alpha1/mixer.proto",
                "proto/music/v1alpha1/playback.proto",
                "proto/music/v1alpha1/playlist.proto",
                "proto/music/v1alpha1/servers.proto",
                "proto/music/v1alpha1/tracklist.proto",
            ],
            &["proto"],
        )?;
    Ok(())
}
