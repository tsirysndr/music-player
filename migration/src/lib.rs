#[cfg(test)]
mod tests;

use music_player_settings::{read_settings, Settings};
pub use sea_orm_migration::prelude::*;
use sea_orm_migration::{cli::run_migrate, sea_orm::Database};
use std::{env, fs::File, path::Path};

mod m20220101_000001_create_table;
mod m20221115_220318_add_folder_table;
mod m20221117_211308_add_created_at_column;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20221115_220318_add_folder_table::Migration),
            Box::new(m20221117_211308_add_created_at_column::Migration),
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
            run_migrate(Migrator, db, None, false).await.unwrap();
        }
        _ => {
            cli::run_cli(Migrator).await;
        }
    }
}
