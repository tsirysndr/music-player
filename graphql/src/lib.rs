use async_graphql::{EmptySubscription, Schema};
use schema::{Mutation, Query};

pub mod schema;

pub type MusicPlayerSchema = Schema<Query, Mutation, EmptySubscription>;
