use anyhow::Error;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use std::collections::HashMap;

pub struct TrackRepository {
    db: DatabaseConnection,
}

impl TrackRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find(&self, id: &str) -> Result<track_entity::Model, Error> {
        let results: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find()
                .filter(track_entity::Column::Id.eq(id))
                .find_with_related(artist_entity::Entity)
                .all(&self.db)
                .await?;
        if results.len() == 0 {
            return Err(Error::msg("Track not found"));
        }
        let track = results[0].0.clone();
        let album =
            album_entity::Entity::find_by_id(track.album_id.unwrap_or_default().to_string())
                .one(&self.db)
                .await?;
        Ok(track_entity::Model {
            artists: results[0].1.clone(),
            album: album.unwrap(),
            id: track.id,
            title: track.title,
            duration: track.duration,
            uri: track.uri,
            artist: track.artist,
            track: track.track,
            ..Default::default()
        })
    }

    pub async fn find_all(
        &self,
        filter: Option<String>,
        offset: Option<u64>,
        limit: u64,
    ) -> Result<Vec<track_entity::Model>, Error> {
        let query = match offset {
            Some(offset) => track_entity::Entity::find().offset(offset).limit(limit),
            None => track_entity::Entity::find().limit(limit),
        };
        let results = match filter {
            Some(filter) => {
                if filter.is_empty() {
                    query
                        .order_by_asc(track_entity::Column::Title)
                        .find_with_related(artist_entity::Entity)
                        .all(&self.db)
                        .await?
                } else {
                    query
                        .filter(track_entity::Column::Title.like(format!("%{}%", filter).as_str()))
                        .order_by_asc(track_entity::Column::Title)
                        .find_with_related(artist_entity::Entity)
                        .all(&self.db)
                        .await?
                }
            }
            None => {
                query
                    .order_by_asc(track_entity::Column::Title)
                    .find_with_related(artist_entity::Entity)
                    .all(&self.db)
                    .await?
            }
        };

        let albums: HashMap<String, album_entity::Model> = album_entity::Entity::find()
            .all(&self.db)
            .await?
            .into_iter()
            .map(|album| (album.id.clone(), album))
            .collect();

        results
            .into_iter()
            .map(|(track, artists)| -> Result<_, Error> {
                let album_id = track
                    .album_id
                    .as_ref()
                    .ok_or_else(|| Error::msg(format!("track {} has no album", track.id)))?;
                let album = albums.get(album_id).cloned().ok_or_else(|| {
                    Error::msg(format!(
                        "track {} references missing album {album_id}",
                        track.id
                    ))
                })?;
                Ok(track_entity::Model {
                    artists,
                    album,
                    ..track
                })
            })
            .collect()
    }
}
