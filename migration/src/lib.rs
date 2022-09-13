use std::{collections::HashMap, fs::File, path::Path};

use music_player_settings::read_settings;
pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20220101_000001_create_table::Migration)]
    }
}

pub async fn run() {
    let settings = read_settings().unwrap();
    let settings = settings
        .try_deserialize::<HashMap<String, String>>()
        .unwrap();
    let database_url = settings.get("database_url").unwrap();

    std::env::set_var("DATABASE_URL", database_url);

    let database_path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite:/", "/");

    if !Path::new(&database_path).exists() {
        File::create(database_path).expect("Failed to create database file");
    }

    cli::run_cli(Migrator).await;
}
