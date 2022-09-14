use std::collections::HashMap;

use music_player_settings::read_settings;
use sea_orm::DatabaseConnection;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
}

impl Database {
    pub async fn new() -> Self {
        let settings = read_settings().unwrap();
        let settings = settings
            .try_deserialize::<HashMap<String, String>>()
            .unwrap();
        let database_url = settings.get("database_url").unwrap();

        let connection = sea_orm::Database::connect(database_url)
            .await
            .expect("Could not connect to database");

        Database { connection }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}
