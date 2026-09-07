use std::sync::Arc;

use async_graphql::*;
use cuid::cuid2;
use futures_util::Stream;
use music_player_addons::CurrentSourceDevice;
use music_player_entity::{
    folder as folder_entity, playlist as playlist_entity,
    playlist_tracks as playlist_tracks_entity, track as track_entity,
};
use music_player_rsql::{QuerySpec, SortOrder as RsqlSortOrder};
use music_player_storage::{repo::playlist::PlaylistRepository, Database};
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, EntityTrait, ModelTrait, QueryFilter, QueryOrder,
};
use tokio::sync::Mutex;

use crate::simple_broker::SimpleBroker;

use super::{
    objects::{folder::Folder, playlist::Playlist, track::Track},
    MutationType,
};

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();

        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let result = source.playlist(&id).await?;
            return Ok(result.into());
        }

        let result = PlaylistRepository::new(db.get_connection())
            .find(id.as_str())
            .await?;

        Ok(result.into())
    }

    async fn playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();

        let current_device = ctx.data::<Arc<Mutex<CurrentSourceDevice>>>().unwrap();
        let mut device = current_device.lock().await;

        if device.client.is_some() {
            let source = device.client.as_mut().unwrap();
            let result = source.playlists(0, 10).await?;
            return Ok(result.into_iter().map(Into::into).collect());
        }

        PlaylistRepository::new(db.get_connection())
            .find_all()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn main_playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();
        PlaylistRepository::new(db.get_connection())
            .main_playlists()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn recent_playlists(&self, ctx: &Context<'_>) -> Result<Vec<Playlist>, Error> {
        let db = ctx.data::<Database>().unwrap();
        PlaylistRepository::new(db.get_connection())
            .recent_playlists()
            .await
            .map(|playlists| playlists.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }

    async fn folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let results: Vec<(folder_entity::Model, Vec<playlist_entity::Model>)> =
            folder_entity::Entity::find_by_id(id.to_string())
                .find_with_related(playlist_entity::Entity)
                .all(db.get_connection())
                .await?;
        if results.len() == 0 {
            return Err(Error::new("Folder not found"));
        }
        let (mut folder, playlists) = results[0].clone();
        folder.playlists = playlists;
        Ok(folder.into())
    }

    /// What a smart-playlist filter would produce, without saving it.
    ///
    /// Cheap enough to call as the user types: it selects ids, then loads only
    /// the handful it shows. A filter that does not compile comes back as an
    /// error carrying the offset of the offending character.
    async fn smart_playlist_preview(
        &self,
        ctx: &Context<'_>,
        smart: SmartPlaylistInput,
        #[graphql(default = 10)] sample: u32,
    ) -> Result<SmartPlaylistPreview, Error> {
        let db = ctx.data::<Database>().unwrap();
        smart.validate()?;
        let ids = music_player_storage::smart_playlist::matching_track_ids(
            db.get_connection(),
            &smart.spec(),
        )
        .await
        .map_err(|e| Error::new(e.to_string()))?;
        let count = ids.len() as u32;

        let head: Vec<String> = ids.into_iter().take(sample as usize).collect();
        let mut tracks: Vec<track_entity::Model> = if head.is_empty() {
            Vec::new()
        } else {
            track_entity::Entity::find()
                .filter(track_entity::Column::Id.is_in(head.clone()))
                .all(db.get_connection())
                .await?
        };
        // `is_in` returns rows in table order; the preview has to show them in
        // the order the filter actually produced.
        tracks.sort_by_key(|track| {
            head.iter()
                .position(|id| *id == track.id)
                .unwrap_or(usize::MAX)
        });

        Ok(SmartPlaylistPreview {
            count,
            tracks: tracks.into_iter().map(Into::into).collect(),
        })
    }

    /// Every field a smart-playlist filter may mention, so a form can offer a
    /// picker rather than expecting the vocabulary to be memorised.
    async fn rsql_fields(&self) -> Vec<RsqlField> {
        music_player_rsql::TRACKS
            .fields
            .iter()
            .map(|field| RsqlField {
                name: field.name.to_owned(),
                label: field.label.to_owned(),
                kind: match field.kind {
                    music_player_rsql::FieldKind::Text => "text",
                    music_player_rsql::FieldKind::Integer => "integer",
                    music_player_rsql::FieldKind::Boolean => "boolean",
                    music_player_rsql::FieldKind::Timestamp => "timestamp",
                }
                .to_owned(),
            })
            .collect()
    }

    async fn folders(&self, ctx: &Context<'_>) -> Result<Vec<Folder>, Error> {
        let db = ctx.data::<Database>().unwrap();
        folder_entity::Entity::find()
            .order_by_asc(folder_entity::Column::Name)
            .all(db.get_connection())
            .await
            .map(|folders| folders.into_iter().map(Into::into).collect())
            .map_err(|e| Error::new(e.to_string()))
    }
}

/// The three things a smart playlist is: a filter, an order, and a cap.
#[derive(Clone, Debug, InputObject)]
pub struct SmartPlaylistInput {
    /// RSQL over the track fields, e.g. `genre==rock;year>2000`. Empty matches
    /// the whole library.
    #[graphql(default)]
    pub filter: String,
    /// A track field to order by, or `random`.
    pub sort_by: Option<String>,
    /// `asc` or `desc`.
    pub sort_order: Option<String>,
    /// Maximum number of tracks. Absent or 0 is unlimited.
    pub limit: Option<u32>,
}

impl SmartPlaylistInput {
    pub fn spec(&self) -> QuerySpec {
        QuerySpec {
            filter: self.filter.clone(),
            sort_by: self.sort_by.clone().filter(|s| !s.trim().is_empty()),
            sort_order: match self.sort_order.as_deref() {
                Some(value) if value.eq_ignore_ascii_case("desc") => RsqlSortOrder::Desc,
                _ => RsqlSortOrder::Asc,
            },
            limit: self.limit,
        }
    }

    /// Check the filter and sort compile, so a bad one is reported against the
    /// form rather than stored and silently matching nothing.
    fn validate(&self) -> Result<(), Error> {
        music_player_rsql::build(&self.spec(), &music_player_rsql::TRACKS)
            .map(|_| ())
            .map_err(|e| Error::new(e.message).extend_with(|_, ext| ext.set("at", e.at as u32)))
    }
}

/// What a filter would produce, without saving anything — the "142 tracks
/// match" line under the filter box, plus the first few by name.
#[derive(SimpleObject)]
pub struct SmartPlaylistPreview {
    /// How many tracks the filter matches.
    pub count: u32,
    /// The first handful, so the form can show what it caught.
    pub tracks: Vec<Track>,
}

/// One filterable field, so a UI can offer a picker instead of making the user
/// remember the vocabulary.
#[derive(SimpleObject)]
pub struct RsqlField {
    /// The name to write in a filter.
    pub name: String,
    /// A human label.
    pub label: String,
    /// `text`, `integer`, `boolean` or `timestamp` — what values it accepts.
    pub kind: String,
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    /// Create a playlist.
    ///
    /// Passing `smart` makes it a smart playlist: the filter is validated and
    /// the tracks generated immediately, so the caller gets back a playlist
    /// that is already populated rather than an empty one to fill by hand.
    async fn create_playlist(
        &self,
        ctx: &Context<'_>,
        name: String,
        description: Option<String>,
        folder_id: Option<ID>,
        smart: Option<SmartPlaylistInput>,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let mut folder: Option<folder_entity::Model> = None;
        let folder_id = match folder_id {
            Some(folder_id) => {
                folder = folder_entity::Entity::find_by_id(folder_id.to_string())
                    .one(db.get_connection())
                    .await?;
                if folder.is_none() {
                    return Err(Error::new("Folder not found"));
                }
                Some(folder_id.to_string())
            }
            None => None,
        };
        // Reject a filter that does not compile *before* creating anything —
        // an empty playlist left behind by a typo is worse than an error.
        if let Some(smart) = &smart {
            smart.validate()?;
        }
        let playlist = playlist_entity::ActiveModel {
            id: ActiveValue::set(cuid2()),
            name: ActiveValue::Set(name),
            description: ActiveValue::Set(description),
            folder_id: ActiveValue::Set(folder_id.clone()),
            created_at: ActiveValue::set(chrono::Utc::now()),
            is_smart: ActiveValue::Set(smart.is_some()),
            rsql: ActiveValue::Set(smart.as_ref().map(|s| s.filter.clone())),
            sort_by: ActiveValue::Set(smart.as_ref().and_then(|s| s.sort_by.clone())),
            sort_order: ActiveValue::Set(smart.as_ref().and_then(|s| s.sort_order.clone())),
            max_tracks: ActiveValue::Set(smart.as_ref().and_then(|s| s.limit)),
            refreshed_at: ActiveValue::Set(None),
        };
        match playlist.insert(db.get_connection()).await {
            Ok(playlist) => {
                if smart.is_some() {
                    music_player_storage::smart_playlist::regenerate(
                        db.get_connection(),
                        &playlist.id,
                    )
                    .await
                    .map_err(|e| Error::new(e.to_string()))?;
                }
                if let Some(folder) = folder {
                    SimpleBroker::publish(FolderChanged {
                        folder: folder.into(),
                        mutation_type: MutationType::Updated,
                        playlist: Some(playlist.clone().into()),
                    });
                }
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Created,
                    track: None,
                });
                Ok(playlist.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    /// Change a smart playlist's filter and regenerate its tracks.
    async fn update_smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        smart: SmartPlaylistInput,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        smart.validate()?;
        let existing = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?
            .ok_or_else(|| Error::new("Playlist not found"))?;
        if !existing.is_smart {
            return Err(Error::new("Not a smart playlist"));
        }
        playlist_entity::ActiveModel {
            id: ActiveValue::Unchanged(id.to_string()),
            rsql: ActiveValue::Set(Some(smart.filter.clone())),
            sort_by: ActiveValue::Set(smart.sort_by.clone()),
            sort_order: ActiveValue::Set(smart.sort_order.clone()),
            max_tracks: ActiveValue::Set(smart.limit),
            ..Default::default()
        }
        .update(db.get_connection())
        .await?;
        self.regenerate_smart_playlist(ctx, id).await
    }

    /// Re-run a smart playlist's filter against the library as it is now.
    async fn regenerate_smart_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        music_player_storage::smart_playlist::regenerate(db.get_connection(), id.as_str())
            .await
            .map_err(|e| Error::new(e.to_string()))?;
        let playlist: Playlist = PlaylistRepository::new(db.get_connection())
            .find(id.as_str())
            .await?
            .into();
        SimpleBroker::publish(PlaylistChanged {
            playlist: playlist.clone(),
            mutation_type: MutationType::Updated,
            track: None,
        });
        Ok(playlist)
    }

    async fn delete_playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;

        match playlist {
            Some(playlist) => {
                playlist.clone().delete(db.get_connection()).await?;
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Deleted,
                    track: None,
                });
                Ok(playlist.into())
            }
            None => Err(Error::new("Playlist not found")),
        }
    }

    async fn add_track_to_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        track_id: ID,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let track = track_entity::Entity::find_by_id(track_id.to_string())
            .one(db.get_connection())
            .await?;
        match track {
            Some(track) => {
                let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                    .one(db.get_connection())
                    .await?;
                match playlist {
                    Some(playlist) => {
                        let playlist_track = playlist_tracks_entity::ActiveModel {
                            id: ActiveValue::set(cuid2()),
                            playlist_id: ActiveValue::Set(playlist.id.clone()),
                            track_id: ActiveValue::Set(track.id.clone()),
                            created_at: ActiveValue::set(chrono::Utc::now()),
                        };
                        playlist_track
                            .insert(db.get_connection())
                            .await
                            .map_err(|err| Error::new(err.to_string()))?;
                        SimpleBroker::publish(PlaylistChanged {
                            playlist: playlist.clone().into(),
                            mutation_type: MutationType::Updated,
                            track: Some(track.clone().into()),
                        });
                        Ok(playlist.into())
                    }
                    None => Err(Error::new("Playlist not found")),
                }
            }
            None => Err(Error::new("Track not found")),
        }
    }

    async fn remove_track_from_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        position: usize,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let playlist_track = playlist_tracks_entity::Entity::find()
            .filter(playlist_tracks_entity::Column::PlaylistId.eq(id.to_string()))
            .all(db.get_connection())
            .await?;
        if playlist_track.len() <= position {
            return Err(Error::new("Track not found"));
        }
        playlist_track[position]
            .clone()
            .delete(db.get_connection())
            .await
            .map_err(|err| Error::new(err.to_string()))?;
        let playlist =
            playlist_entity::Entity::find_by_id(playlist_track[position].clone().playlist_id)
                .one(db.get_connection())
                .await?;
        SimpleBroker::publish(PlaylistChanged {
            playlist: playlist.clone().unwrap().into(),
            mutation_type: MutationType::Updated,
            track: None,
        });
        Ok(playlist.unwrap().into())
    }

    async fn rename_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Playlist, Error> {
        let db = ctx.data::<Database>().unwrap();
        let playlist = playlist_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut playlist: playlist_entity::ActiveModel = match playlist {
            Some(playlist) => playlist.into(),
            None => return Err(Error::new("Playlist not found")),
        };
        playlist.name = ActiveValue::Set(name);
        match playlist.update(db.get_connection()).await {
            Ok(playlist) => {
                SimpleBroker::publish(PlaylistChanged {
                    playlist: playlist.clone().into(),
                    mutation_type: MutationType::Renamed,
                    track: None,
                });
                Ok(playlist.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn create_folder(&self, ctx: &Context<'_>, name: String) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::ActiveModel {
            id: ActiveValue::set(cuid2()),
            name: ActiveValue::Set(name),
            created_at: ActiveValue::set(chrono::Utc::now()),
        };
        match folder.insert(db.get_connection()).await {
            Ok(folder) => {
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Created,
                    playlist: None,
                });
                Ok(folder.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn delete_folder(&self, ctx: &Context<'_>, id: ID) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                folder.clone().delete(db.get_connection()).await?;
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Deleted,
                    playlist: None,
                });
                Ok(folder.into())
            }
            None => Err(Error::new("Folder not found")),
        }
    }

    async fn rename_folder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::Entity::find_by_id(id.to_string())
            .one(db.get_connection())
            .await?;
        let mut folder: folder_entity::ActiveModel = match folder {
            Some(folder) => folder.into(),
            None => return Err(Error::new("Folder not found")),
        };
        folder.name = ActiveValue::Set(name);
        match folder.update(db.get_connection()).await {
            Ok(folder) => {
                SimpleBroker::publish(FolderChanged {
                    folder: folder.clone().into(),
                    mutation_type: MutationType::Renamed,
                    playlist: None,
                });
                Ok(folder.into())
            }
            Err(err) => Err(Error::new(err.to_string())),
        }
    }

    async fn move_playlist_to_folder(
        &self,
        ctx: &Context<'_>,
        id: ID,
        folder_id: ID,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();
        let folder = folder_entity::Entity::find_by_id(folder_id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                    .one(db.get_connection())
                    .await?;
                match playlist {
                    Some(playlist) => {
                        let moved_playlist = playlist.clone();
                        let mut playlist: playlist_entity::ActiveModel = playlist.into();
                        playlist.folder_id = ActiveValue::Set(Some(folder.id.clone()));
                        match playlist.update(db.get_connection()).await {
                            Ok(_) => {
                                SimpleBroker::publish(PlaylistChanged {
                                    playlist: moved_playlist.into(),
                                    mutation_type: MutationType::Moved,
                                    track: None,
                                });
                                SimpleBroker::publish(FolderChanged {
                                    folder: folder.clone().into(),
                                    mutation_type: MutationType::Updated,
                                    playlist: None,
                                });
                                Ok(folder.into())
                            }
                            Err(err) => Err(Error::new(err.to_string())),
                        }
                    }
                    None => Err(Error::new("Playlist not found")),
                }
            }
            None => Err(Error::new("Folder not found")),
        }
    }

    async fn move_playlists_to_folder(
        &self,
        ctx: &Context<'_>,
        ids: Vec<ID>,
        folder_id: ID,
    ) -> Result<Folder, Error> {
        let db = ctx.data::<Database>().unwrap();

        let folder = folder_entity::Entity::find_by_id(folder_id.to_string())
            .one(db.get_connection())
            .await?;
        match folder {
            Some(folder) => {
                for id in ids {
                    let playlist = playlist_entity::Entity::find_by_id(id.to_string())
                        .one(db.get_connection())
                        .await?;
                    match playlist {
                        Some(playlist) => {
                            let moved_playlist = playlist.clone();
                            let mut playlist: playlist_entity::ActiveModel = playlist.into();
                            playlist.folder_id = ActiveValue::Set(Some(folder.id.clone()));
                            let p = playlist.update(db.get_connection()).await?;
                            SimpleBroker::publish(PlaylistChanged {
                                playlist: moved_playlist.into(),
                                mutation_type: MutationType::Moved,
                                track: None,
                            });
                            Ok(p)
                        }
                        None => Err(Error::new("Playlist not found")),
                    }?;
                }
                Ok(folder.into())
            }
            None => Err(Error::new("Folder not found")),
        }
    }
}

#[derive(Clone)]
struct FolderChanged {
    folder: Folder,
    mutation_type: MutationType,
    playlist: Option<Playlist>,
}

#[Object]
impl FolderChanged {
    async fn folder(&self) -> &Folder {
        &self.folder
    }

    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn playlist(&self) -> Option<&Playlist> {
        self.playlist.as_ref()
    }
}

#[derive(Clone)]
struct PlaylistChanged {
    playlist: Playlist,
    mutation_type: MutationType,
    track: Option<Track>,
}

#[Object]
impl PlaylistChanged {
    async fn playlist(&self) -> &Playlist {
        &self.playlist
    }

    async fn mutation_type(&self) -> MutationType {
        self.mutation_type
    }

    async fn track(&self) -> Option<&Track> {
        self.track.as_ref()
    }
}

#[derive(Default)]
pub struct PlaylistSubscription;

#[Subscription]
impl PlaylistSubscription {
    async fn playlists(&self) -> impl Stream<Item = Vec<Playlist>> {
        SimpleBroker::<Vec<Playlist>>::subscribe()
    }

    async fn playlist(&self, _id: ID) -> impl Stream<Item = PlaylistChanged> {
        SimpleBroker::<PlaylistChanged>::subscribe()
    }

    async fn folders(&self) -> impl Stream<Item = Vec<Folder>> {
        SimpleBroker::<Vec<Folder>>::subscribe()
    }

    async fn folder(&self, _id: ID) -> impl Stream<Item = FolderChanged> {
        SimpleBroker::<FolderChanged>::subscribe()
    }
}
