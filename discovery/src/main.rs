use mdns::Error;
use music_player_discovery::{discover, register_services, SERVICE_NAME};

#[tokio::main]
async fn main() -> Result<(), Error> {
    //register_services();
    discover(SERVICE_NAME);
    Ok(())
}
