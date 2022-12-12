#[cfg(test)]
mod tests;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{
    error::ErrorNotFound,
    guard,
    http::header::HOST,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use fs::NamedFile;
use mime_guess::from_path;
use music_player_entity::track as track_entity;
use music_player_graphql::{
    scan_devices,
    schema::{Mutation, Query, Subscription},
    MusicPlayerSchema,
};
use music_player_playback::player::PlayerCommand;
use music_player_settings::{read_settings, Settings};
use music_player_storage::Database;
use music_player_tracklist::Tracklist;
use owo_colors::OwoColorize;
use rust_embed::RustEmbed;
use sea_orm::EntityTrait;
use std::{path::PathBuf, sync::Arc, thread};
use tokio::sync::{mpsc::UnboundedSender, Mutex};
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

async fn index_spa() -> impl Responder {
    handle_embedded_file("index.html")
}

async fn index_file(db: Data<Arc<Mutex<Database>>>, req: HttpRequest) -> Result<NamedFile, Error> {
    let id = req.match_info().get("id").unwrap();
    let mut path = PathBuf::new();

    let track = track_entity::Entity::find_by_id(id.to_owned())
        .one(db.lock().await.get_connection())
        .await
        .unwrap();

    if let Some(track) = track {
        path.push(track.uri);
        Ok(NamedFile::open(path)?)
    } else {
        Err(ErrorNotFound("Track not found".to_string()))
    }
}

async fn index_ws(
    schema: web::Data<MusicPlayerSchema>,
    req: HttpRequest,
    payload: web::Payload,
) -> Result<HttpResponse> {
    GraphQLSubscription::new(Schema::clone(&*schema)).start(&req, payload)
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
    let host = req
        .headers()
        .get(HOST)
        .unwrap()
        .to_str()
        .unwrap()
        .split(":")
        .next()
        .unwrap();

    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let graphql_endpoint = format!("http://{}:{}/graphql", host, settings.http_port);
    let ws_endpoint = format!("ws://{}:{}/graphql", host, settings.http_port);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint(&graphql_endpoint)
                .subscription_endpoint(&ws_endpoint)
                .finish(),
        ))
}

#[actix_web::get("/{_:.*}")]
async fn dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}

pub async fn start_webui(
    cmd_tx: Arc<std::sync::Mutex<UnboundedSender<PlayerCommand>>>,
    tracklist: Arc<std::sync::Mutex<Tracklist>>,
) -> std::io::Result<()> {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let addr = format!("0.0.0.0:{}", settings.http_port);

    let db = Arc::new(Mutex::new(Database::new().await));

    let devices = scan_devices().await.unwrap();

    let schema = Schema::build(
        Query::default(),
        Mutation::default(),
        Subscription::default(),
    )
    .data(Arc::clone(&db))
    .data(cmd_tx)
    .data(tracklist)
    .data(devices)
    .finish();
    println!("Starting webui at {}", addr.bright_green());

    HttpServer::new(move || {
        let cors = Cors::permissive();

        let covers_path = format!(
            "{}/music-player/covers",
            dirs::config_dir().unwrap().to_str().unwrap()
        );
        App::new()
            .app_data(Data::new(Arc::clone(&db)))
            .app_data(Data::new(schema.clone()))
            .wrap(cors)
            .service(index_graphql)
            .service(index_graphiql)
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
            .service(fs::Files::new("/covers", covers_path).show_files_listing())
            .service(index)
            .route("/tracks", web::get().to(index_spa))
            .route("/artists", web::get().to(index_spa))
            .route("/albums", web::get().to(index_spa))
            .route("/artists/{_:.*}", web::get().to(index_spa))
            .route("/albums/{_:.*}", web::get().to(index_spa))
            .route("/folders/{_:.*}", web::get().to(index_spa))
            .route("/playlists/{_:.*}", web::get().to(index_spa))
            .route("/search", web::get().to(index_spa))
            .route("/tracks/{id}", web::get().to(index_file))
            .service(dist)
    })
    .bind(addr)?
    .run()
    .await
}
