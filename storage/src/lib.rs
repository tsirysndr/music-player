use searcher::Searcher;

use music_player_settings::{read_settings, Settings};
use sea_orm::DatabaseConnection;

pub mod searcher;

#[derive(Clone)]
pub struct Database {
    pub connection: DatabaseConnection,
    pub searcher: Searcher,
}

impl Database {
    pub async fn new() -> Database {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();

        let connection = sea_orm::Database::connect(settings.database_url)
            .await
            .expect("Could not connect to database");

        let searcher = Searcher::new();

        Database {
            connection,
            searcher,
        }
    }

    pub fn get_connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn get_searcher(&self) -> &Searcher {
        &self.searcher
    }
}
