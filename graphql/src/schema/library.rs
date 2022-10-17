use async_graphql::*;

use super::objects::track::Track;

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
}

pub struct LibraryMutation;

impl LibraryMutation {}
