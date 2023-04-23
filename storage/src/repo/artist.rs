use anyhow::Error;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

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
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<artist_entity::Model>, Error> {
        let mut query = match offset {
            Some(offset) => artist_entity::Entity::find()
                .order_by_asc(artist_entity::Column::Name)
                .offset(offset),
            None => artist_entity::Entity::find().order_by_asc(artist_entity::Column::Name),
        };

        query = match limit {
            Some(limit) => query.limit(limit),
            None => query,
        };

        match filter {
            Some(filter) => {
                if filter.is_empty() {
                    let results = query.all(&self.db).await?;
                    return Ok(results);
                }
                let results = query
                    .filter(artist_entity::Column::Name.like(format!("%{}%", filter).as_str()))
                    .all(&self.db)
                    .await?;
                Ok(results)
            }
            None => {
                let results = query.all(&self.db).await?;
                Ok(results)
            }
        }
    }
}
