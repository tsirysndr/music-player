use futures_util::StreamExt;
use mdns::Error;
use music_player_discovery::{discover, register_services, MdnsResponder, SERVICE_NAME};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut responder = MdnsResponder::new();
    responder.register_service("service1", 8080);
    responder.register_service("service2", 8080);
    responder.register_service("service3", 8080);
    let services = discover(SERVICE_NAME);
    tokio::pin!(services);
    while let Some(srv) = services.next().await {
        println!("got = {:?}", srv);
    }
    Ok(())
}
