use async_graphql::*;

#[derive(Clone)]
pub struct Album {
    pub id: ID,
    pub title: String,
}

#[Object]
impl Album {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn title(&self) -> &str {
        &self.title
    }
}
