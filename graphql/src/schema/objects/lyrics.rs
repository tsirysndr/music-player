use async_graphql::*;

#[derive(Default, Clone)]
pub struct Lyrics {
    pub id: ID,
    pub content: String,
}

#[Object]
impl Lyrics {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn content(&self) -> &str {
        &self.content
    }
}
