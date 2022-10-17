use async_graphql::*;

#[derive(InputObject, Clone)]
pub struct TrackInput {
    pub id: ID,
    pub title: String,
}

#[derive(Clone)]
pub struct Track {
    pub id: ID,
    pub title: String,
}

#[Object]
impl Track {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }
}
