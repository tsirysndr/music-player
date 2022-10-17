use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder, Result,
};
use async_graphql::{http::GraphiQLSource, EmptyMutation, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use mime_guess::from_path;
use music_player_graphql::{
    schema::{Mutation, Query},
    MusicPlayerSchema,
};
use owo_colors::OwoColorize;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "musicplayer/build/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::get("/")]
async fn index() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::post("/graphql")]
async fn index_graphql(
    schema: web::Data<MusicPlayerSchema>,
    req: GraphQLRequest,
) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::get("/graphiql")]
async fn index_graphiql() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint("http://localhost:8000/graphql")
                .finish(),
        ))
}

#[actix_web::get("/{_:.*}")]
async fn dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::new(Query::default(), Mutation::default(), EmptySubscription);
    println!(
        "Starting webui at {}",
        "http://localhost:8000".bright_green()
    );
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(index_graphql)
            .service(index_graphiql)
            .service(index)
            .service(dist)
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
