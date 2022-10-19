use async_graphql::*;

#[derive(Default, Clone)]
pub struct Artist {
    pub id: ID,
    pub name: String,
}

#[Object]
impl Artist {
    async fn id(&self) -> &str {
        &self.id
    }

    async fn name(&self) -> &str {
        &self.name
    }
}
