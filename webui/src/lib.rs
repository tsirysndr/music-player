use actix_files as fs;
use actix_web::{
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use mime_guess::from_path;
use music_player_graphql::{
    schema::{Mutation, Query},
    MusicPlayerSchema,
};
use music_player_settings::{read_settings, Settings};
use owo_colors::OwoColorize;
use rust_embed::RustEmbed;
use serde_derive::Deserialize;

#[derive(Deserialize)]
struct Params {
    pub origin: Option<String>,
}

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
async fn index_graphiql(req: HttpRequest) -> Result<HttpResponse> {
    let params = web::Query::<Params>::from_query(req.query_string()).unwrap();
    let origin = params.origin.clone();
    let host = origin.unwrap_or_else(|| "localhost".to_string());
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let graphql_endpoint = format!("http://{}:{}/graphql", host, settings.http_port);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(GraphiQLSource::build().endpoint(&graphql_endpoint).finish()))
}

#[actix_web::get("/{_:.*}")]
async fn dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

pub async fn start_server() -> std::io::Result<()> {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let addr = format!("0.0.0.0:{}", settings.http_port);

    let schema = Schema::new(Query::default(), Mutation::default(), EmptySubscription);
    println!("Starting webui at {}", addr.bright_green());

    HttpServer::new(move || {
        let covers_path = format!(
            "{}/music-player/covers",
            dirs::config_dir().unwrap().to_str().unwrap()
        );
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(index_graphql)
            .service(index_graphiql)
            .service(fs::Files::new("/covers", covers_path).show_files_listing())
            .service(index)
            .service(dist)
    })
    .bind(addr)?
    .run()
    .await
}
