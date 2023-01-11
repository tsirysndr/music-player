use anyhow::Error;
use music_player_entity::{album as album_entity, artist as artist_entity, track as track_entity};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect};

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

    pub async fn find_all(&self, limit: u64) -> Result<Vec<track_entity::Model>, Error> {
        let results: Vec<(track_entity::Model, Vec<artist_entity::Model>)> =
            track_entity::Entity::find()
                .limit(limit)
                .order_by_asc(track_entity::Column::Title)
                .find_with_related(artist_entity::Entity)
                .all(&self.db)
                .await?;

        let albums: Vec<(track_entity::Model, Option<album_entity::Model>)> =
            track_entity::Entity::find()
                .limit(limit)
                .order_by_asc(track_entity::Column::Title)
                .find_also_related(album_entity::Entity)
                .all(&self.db)
                .await?;

        let albums: Vec<Option<album_entity::Model>> = albums
            .into_iter()
            .map(|(_track, album)| album.clone())
            .collect();
        let mut albums = albums.into_iter();

        Ok(results
            .into_iter()
            .map(|(track, artists)| {
                let album = albums.next().unwrap().unwrap();
                track_entity::Model {
                    artists,
                    album,
                    ..track
                }
            })
            .collect())
    }
}
