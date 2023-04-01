#[cfg(test)]
mod tests;

use music_player_settings::{read_settings, Settings};
use sea_orm::{ConnectOptions, DatabaseConnection};

pub mod searcher;

pub mod repo;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Database {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let mut opt = ConnectOptions::new(settings.database_url);
        opt.max_connections(100).min_connections(5);
        let connection = sea_orm::Database::connect(opt)
            .await
            .expect("Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
