use async_graphql::*;

use super::objects::playlist::Playlist;

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        todo!()
    }

    async fn playlists(&self, ctx: &Context<'_>) -> Result<bool, Error> {
        todo!()
    }

    async fn get_playlist_tracks(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        todo!()
    }
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn create_playlist(&self, ctx: &Context<'_>, id: ID) -> Result<Playlist, Error> {
        todo!()
    }

    async fn delete_playlist(&self, ctx: &Context<'_>, id: ID) -> Result<bool, Error> {
        todo!()
    }

    async fn add_track_to_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        track_id: ID,
    ) -> Result<Playlist, Error> {
        todo!()
    }

    async fn remove_track_from_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        track_id: ID,
    ) -> Result<Playlist, Error> {
        todo!()
    }

    async fn rename_playlist(
        &self,
        ctx: &Context<'_>,
        id: ID,
        name: String,
    ) -> Result<Playlist, Error> {
        todo!()
    }

    async fn play_playlist(&self, ctx: &Context<'_>, id: ID, shuffle: bool) -> Result<bool, Error> {
        todo!()
    }
}
