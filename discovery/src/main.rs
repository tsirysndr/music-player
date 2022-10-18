use mdns::Error;
use music_player_discovery::{discover, register_services, MdnsResponder, SERVICE_NAME};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut responder = MdnsResponder::new();
    responder.register_service("service1", 8080);
    responder.register_service("service2", 8080);
    responder.register_service("service3", 8080);
    discover(SERVICE_NAME);
    Ok(())
}
