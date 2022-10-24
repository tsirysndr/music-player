use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use music_player_graphql::schema::{Mutation, Query};
use tide::{http::mime, Body, Response, StatusCode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema = Schema::new(Query::default(), Mutation::default(), EmptySubscription);
    let mut app = tide::new();

    app.at("/graphql").post(async_graphql_tide::graphql(schema));

    app.at("/").get(|_| async move {
        let mut resp = Response::new(StatusCode::Ok);
        resp.set_body(Body::from_string(
            GraphiQLSource::build().endpoint("/graphql").finish(),
        ));
        resp.set_content_type(mime::HTML);
        Ok(resp)
    });

    app.listen("0.0.0.0:5053").await?;

    Ok(())
}
