use async_graphql::*;

use super::objects::track::Track;

#[derive(Default)]
pub struct LibraryQuery;

#[Object]
impl LibraryQuery {
    async fn tracks(&self, ctx: &Context<'_>) -> Vec<Track> {
        vec![]
    }

    async fn artists(&self, ctx: &Context<'_>) -> Vec<String> {
        vec![]
    }

    async fn albums(&self, ctx: &Context<'_>) -> Vec<String> {
        vec![]
    }

    async fn search(&self, ctx: &Context<'_>) -> bool {
        false
    }
}

#[derive(Default)]
pub struct LibraryMutation;

#[Object]
impl LibraryMutation {
    async fn scan(&self, ctx: &Context<'_>) -> bool {
        false
    }
}
