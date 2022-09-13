use std::{fs::File, path::Path};

use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    let database_path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite://", "");
    if !Path::new(&database_path).exists() {
        File::create(database_path).expect("Failed to create database file");
    }
    cli::run_cli(migration::Migrator).await;
}
