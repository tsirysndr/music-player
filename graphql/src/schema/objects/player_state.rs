use async_graphql::*;

#[derive(Default, Clone)]
pub struct PlayerState {
    pub index: u32,
    pub position_ms: u32,
    pub is_playing: bool,
}

#[Object]
impl PlayerState {
    async fn index(&self) -> u32 {
        self.index
    }

    async fn position_ms(&self) -> u32 {
        self.position_ms
    }

    async fn is_playing(&self) -> bool {
        self.is_playing
    }
}
