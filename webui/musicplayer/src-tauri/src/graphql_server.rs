use std::io::Error;

use actix_web::{
    guard,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse, GraphQLSubscription};
use music_player_graphql::MusicPlayerSchema;
use music_player_settings::{read_settings, Settings};

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
    let connection_info = req.connection_info();
    let host =  connection_info.host();
    let graphql_endpoint = format!("http://{}/graphql", host);
    let ws_endpoint = format!("ws://{}/graphql", host);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint(&graphql_endpoint)
                .subscription_endpoint(&ws_endpoint)
                .finish(),
        ))
}

#[actix_web::get("/")]
async fn index() -> Result<HttpResponse> {
    Ok(HttpResponse::TemporaryRedirect()
        .append_header(("Location", "/graphiql"))
        .finish())
}

pub async fn run_graphql_server(schema: MusicPlayerSchema) -> Result<(), Error> {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let addr = format!("0.0.0.0:{}", settings.http_port);

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(schema.clone()))
            .service(index_graphql)
            .service(index_graphiql)
            .service(index)
            .service(
                web::resource("/graphql")
                    .guard(guard::Get())
                    .guard(guard::Header("upgrade", "websocket"))
                    .to(index_ws),
            )
        })
        .bind(addr).unwrap()
        .run();
    server.await
}
