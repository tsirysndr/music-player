use async_graphql::Schema;
use schema::{Mutation, Query, Subscription};

pub mod schema;
pub mod simple_broker;

pub type MusicPlayerSchema = Schema<Query, Mutation, Subscription>;
