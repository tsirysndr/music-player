use anyhow::Error;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter, QueryOrder, QuerySelect,
};

pub struct AlbumRepository {
    db: DatabaseConnection,
}

impl AlbumRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find(&self, id: &str) -> Result<album_entity::Model, Error> {
        let result = album_entity::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?;
        if result.is_none() {
            return Err(Error::msg("Album not found"));
        }
        let mut album = result.unwrap();
        let mut tracks = album
            .find_related(track_entity::Entity)
            .order_by_asc(track_entity::Column::Track)
            .all(&self.db)
            .await?;
        for track in &mut tracks {
            track.artists = track
                .find_related(artist_entity::Entity)
                .all(&self.db)
                .await?;
            track.album = album.clone();
        }
        album.tracks = tracks;
        Ok(album)
    }

    pub async fn find_all(
        &self,
        filter: Option<String>,
        offset: Option<u64>,
        limit: Option<u64>,
    ) -> Result<Vec<album_entity::Model>, Error> {
        let mut query = match offset {
            Some(offset) => album_entity::Entity::find()
                .order_by_asc(album_entity::Column::Title)
                .offset(offset),
            None => album_entity::Entity::find().order_by_asc(album_entity::Column::Title),
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
                    .filter(album_entity::Column::Title.like(format!("%{}%", filter).as_str()))
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
