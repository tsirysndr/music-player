#[cfg(test)]
mod tests;

use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use music_player_settings::{read_settings, Settings};
use sea_orm::{sea_query, ConnectOptions, ConnectionTrait, DatabaseConnection};

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

    pub async fn create_indexes(&self) {
        let builder = self.connection.get_database_backend();
        let track_title_idx = sea_query::Index::create()
            .table(track_entity::Entity)
            .name("track_title_index")
            .col(track_entity::Column::Title)
            .to_owned();
        let sql = builder.build(&track_title_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("track_title_index already exists, skipping");
            }
        }

        let album_title_idx = sea_query::Index::create()
            .table(album_entity::Entity)
            .name("album_title_index")
            .col(album_entity::Column::Title)
            .to_owned();
        let sql = builder.build(&album_title_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("album_title_index already exists, skipping");
            }
        }

        let artist_name_idx = sea_query::Index::create()
            .table(artist_entity::Entity)
            .name("artist_name_index")
            .col(artist_entity::Column::Name)
            .to_owned();
        let sql = builder.build(&artist_name_idx);
        match self.connection.execute(sql).await {
            Ok(_) => {}
            Err(_) => {
                println!("artist_name_index already exists, skipping");
            }
        }
    }
}
