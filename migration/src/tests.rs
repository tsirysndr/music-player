use std::{env, fs::File, path::Path};

use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, DatabaseBackend, QueryResult, Statement},
};

#[tokio::test]
async fn test_cli() {
    env::set_var("DATABASE_URL", "sqlite:///tmp/test.db");
    let database_path = std::env::var("DATABASE_URL")
        .unwrap()
        .replace("sqlite://", "");
    if !Path::new(&database_path).exists() {
        File::create(database_path).expect("Failed to create database file");
    }

    cli::run_cli(super::Migrator).await;

    let db = sea_orm::Database::connect("sqlite:///tmp/test.db")
        .await
        .expect("Could not connect to database");

    let result: Vec<QueryResult> = db
        .query_all(Statement::from_string(
            DatabaseBackend::Sqlite,
            "SELECT name FROM sqlite_master
    WHERE type='table';"
                .to_owned(),
        ))
        .await
        .unwrap();

    assert_eq!(result.len(), 9);
}
