use async_graphql::*;

#[derive(Default)]
pub struct PlaylistQuery;

#[Object]
impl PlaylistQuery {
    async fn playlist(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn playlists(&self, ctx: &Context<'_>) -> bool {
        false
    }

    async fn get_playlist_tracks(&self, ctx: &Context<'_>, id: ID) -> bool {
        false
    }
}

#[derive(Default)]
pub struct PlaylistMutation;

#[Object]
impl PlaylistMutation {
    async fn create_playlist(&self, ctx: &Context<'_>, id: ID) -> bool {
        false
    }

    async fn delete_playlist(&self, ctx: &Context<'_>, id: ID) -> bool {
        false
    }

    async fn add_track_to_playlist(&self, ctx: &Context<'_>, id: ID, track_id: ID) -> bool {
        false
    }

    async fn remove_track_from_playlist(&self, ctx: &Context<'_>, id: ID, track_id: ID) -> bool {
        false
    }

    async fn rename_playlist(&self, ctx: &Context<'_>, id: ID, name: String) -> bool {
        false
    }

    async fn play_playlist(&self, ctx: &Context<'_>, id: ID, shuffle: bool) -> bool {
        false
    }
}
