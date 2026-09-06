fn main() -> Result<(), Box<dyn std::error::Error>> {
    // gRPC clients come from music-player-server's generated API — no proto
    // codegen here, only the Slint UI compile.
    slint_build::compile_with_config(
        "ui/app.slint",
        slint_build::CompilerConfiguration::new().with_style("fluent-dark".into()),
    )?;
    Ok(())
}
