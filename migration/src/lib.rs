#[cfg(test)]
mod tests;

use music_player_settings::{read_settings, Settings};
pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::Database;
use std::{env, fs::File, path::Path};

mod m20220101_000001_create_table;
mod m20221115_220318_add_folder_table;
mod m20221117_211308_add_created_at_column;
mod m20260905_000001_create_search_index;
mod m20260906_000001_add_artist_picture;
mod m20260906_000002_create_saved_radio;

/// Create the database file if needed and bring the schema up to date,
/// without going through the sea-orm migration CLI (which parses argv).
pub async fn apply() {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    env::set_var("DATABASE_URL", &settings.database_url);
    let db_path = settings
        .database_url
        .replace("sqlite://", "")
        .replace("sqlite:", "");
    if !Path::new(&db_path).exists() {
        File::create(&db_path).unwrap();
    }
    let db = Database::connect(&settings.database_url).await.unwrap();
    Migrator::up(&db, None).await.unwrap();
}

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20221115_220318_add_folder_table::Migration),
            Box::new(m20221117_211308_add_created_at_column::Migration),
            Box::new(m20260905_000001_create_search_index::Migration),
            Box::new(m20260906_000001_add_artist_picture::Migration),
            Box::new(m20260906_000002_create_saved_radio::Migration),
        ]
    }
}

pub async fn run() {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let url = settings.database_url.clone();

    std::env::set_var("DATABASE_URL", settings.database_url);

    let database_path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite:", "");

    if !Path::new(&database_path).exists() {
        File::create(database_path).expect("Failed to create database file");
    }

    match env::consts::OS {
        "android" => {
            let db = &Database::connect(&url).await.unwrap();
            Migrator::up(&db, None).await.unwrap();
        }
        _ => {
            cli::run_cli(Migrator).await;
        }
    }
}
