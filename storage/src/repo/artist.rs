use anyhow::Error;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};

pub struct ArtistRepository {
    db: DatabaseConnection,
}

impl ArtistRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find(&self, id: &str) -> Result<artist_entity::Model, Error> {
        let result = artist_entity::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?;

        if result.is_none() {
            return Err(Error::msg("Artist not found"));
        }

        let mut artist = result.unwrap();
        let results: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::ArtistId.eq(id.clone()))
                .order_by_asc(track_entity::Column::Title)
                .find_also_related(album_entity::Entity)
                .all(&self.db)
                .await?;

        artist.tracks = results
            .into_iter()
            .map(|(track, album)| {
                let mut track = track;
                track.artists = vec![artist.clone()];
                track.album = album.unwrap();
                track
            })
            .collect();

        artist.albums = album_entity::Entity::find()
            .filter(album_entity::Column::ArtistId.eq(id.clone()))
            .order_by_asc(album_entity::Column::Title)
            .all(&self.db)
            .await?;
        Ok(artist)
    }

    pub async fn find_all(
        &self,
        filter: Option<String>,
    ) -> Result<Vec<artist_entity::Model>, Error> {
        match filter {
            Some(filter) => {
                let results = artist_entity::Entity::find()
                    .filter(artist_entity::Column::Name.like(format!("%{}%", filter).as_str()))
                    .order_by_asc(artist_entity::Column::Name)
                    .all(&self.db)
                    .await?;
                Ok(results)
            }
            None => {
                let results = artist_entity::Entity::find()
                    .order_by_asc(artist_entity::Column::Name)
                    .all(&self.db)
                    .await?;
                Ok(results)
            }
        }
    }
}
