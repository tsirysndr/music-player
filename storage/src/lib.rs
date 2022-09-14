use std::collections::HashMap;

use music_player_settings::{read_settings, Settings};
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let connection = sea_orm::Database::connect(settings.database_url)
            .await
            .expect("Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
