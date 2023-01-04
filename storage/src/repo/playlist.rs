use anyhow::Error;
use music_player_entity::{
    album as album_entity, artist as artist_entity, playlist as playlist_entity,
    playlist_tracks as playlist_tracks_entity, select_result, track as track_entity,
};
use music_player_types::types::Playlist;
use sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QueryOrder, QuerySelect,
    RelationTrait,
};

pub struct PlaylistRepository {
    db: DatabaseConnection,
}

impl PlaylistRepository {
    pub fn new(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }

    pub async fn find(&self, id: &str) -> Result<Playlist, Error> {
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?;
        if playlist.is_none() {
            return Err(Error::msg("Playlist not found"));
        }
        let result = match playlist_tracks_entity::Entity::find()
            .filter(playlist_tracks_entity::Column::PlaylistId.eq(id.to_string()))
            .one(&self.db)
            .await?
        {
            Some(_) => {
                let results = playlist_entity::Entity::find_by_id(id.to_string())
                    .select_only()
                    .column(playlist_entity::Column::Id)
                    .column(playlist_entity::Column::Name)
                    .column(playlist_entity::Column::Description)
                    .column_as(artist_entity::Column::Id, "artist_id")
                    .column_as(artist_entity::Column::Name, "artist_name")
                    .column_as(album_entity::Column::Id, "album_id")
                    .column_as(album_entity::Column::Title, "album_title")
                    .column_as(album_entity::Column::Cover, "album_cover")
                    .column_as(album_entity::Column::Year, "album_year")
                    .column_as(track_entity::Column::Id, "track_id")
                    .column_as(track_entity::Column::Title, "track_title")
                    .column_as(track_entity::Column::Duration, "track_duration")
                    .column_as(track_entity::Column::Track, "track_number")
                    .column_as(track_entity::Column::Artist, "track_artist")
                    .column_as(track_entity::Column::Uri, "track_uri")
                    .column_as(track_entity::Column::Genre, "track_genre")
                    .join_rev(
                        JoinType::LeftJoin,
                        playlist_tracks_entity::Entity::belongs_to(playlist_entity::Entity)
                            .from(playlist_tracks_entity::Column::PlaylistId)
                            .to(playlist_entity::Column::Id)
                            .into(),
                    )
                    .join(
                        JoinType::LeftJoin,
                        playlist_tracks_entity::Relation::Track.def(),
                    )
                    .join(JoinType::LeftJoin, track_entity::Relation::Album.def())
                    .join(JoinType::LeftJoin, track_entity::Relation::Artist.def())
                    .into_model::<select_result::PlaylistTrack>()
                    .all(&self.db)
                    .await?;

                if results.len() == 0 {
                    return Err(Error::msg("Playlist not found"));
                }
                results
            }
            None => {
                let result = playlist_entity::Entity::find_by_id(id.to_string())
                    .one(&self.db)
                    .await?;
                if result.is_none() {
                    return Err(Error::msg("Playlist not found"));
                }
                vec![]
            }
        };
        let playlist = playlist.unwrap();
        Ok(Playlist {
            id: playlist.id,
            name: playlist.name,
            description: playlist.description,
            tracks: result.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn find_all(&self) -> Result<Vec<Playlist>, Error> {
        playlist_entity::Entity::find()
            .order_by_asc(playlist_entity::Column::Name)
            .all(&self.db)
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::msg(e.to_string()))
    }

    pub async fn main_playlists(&self) -> Result<Vec<Playlist>, Error> {
        playlist_entity::Entity::find()
            .order_by_asc(playlist_entity::Column::Name)
            .filter(playlist_entity::Column::FolderId.is_null())
            .all(&self.db)
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::msg(e.to_string()))
    }

    pub async fn recent_playlists(&self) -> Result<Vec<Playlist>, Error> {
        playlist_entity::Entity::find()
            .order_by_desc(playlist_entity::Column::CreatedAt)
            .limit(10)
            .all(&self.db)
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::msg(e.to_string()))
    }
}
