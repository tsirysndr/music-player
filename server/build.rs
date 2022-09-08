fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(
        &[
            "proto/music/v1alpha1/core.proto",
            "proto/music/v1alpha1/history.proto",
            "proto/music/v1alpha1/library.proto",
            "proto/music/v1alpha1/mixer.proto",
            "proto/music/v1alpha1/playback.proto",
            "proto/music/v1alpha1/playlist.proto",
            "proto/music/v1alpha1/tracklist.proto",
        ],
        &["proto"],
    )?;
    Ok(())
}
