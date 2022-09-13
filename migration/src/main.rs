use std::fs::File;

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    File::create(
        std::env::var("DATABASE_URL")
            .unwrap()
            .replace("sqlite://", ""),
    )
    .expect("Failed to create database file");
    cli::run_cli(migration::Migrator).await;
}
