use futures_util::StreamExt;
use mdns_sd::ServiceInfo;
use music_player_settings::{read_settings, Settings};

use crate::SERVICE_NAME;

#[tokio::test]
async fn discover() {
    super::register_services();
    let services = super::discover(SERVICE_NAME);
    tokio::pin!(services);
    let mut discovered: Vec<ServiceInfo> = vec![];
    while let Some(info) = services.next().await {
        discovered.push(info);
        if discovered.len() == 3 {
            break;
        }
    }

    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();

    let grpc_service = format!("grpc-{}.{}", settings.device_id, SERVICE_NAME);
    let http_service = format!("http-{}.{}", settings.device_id, SERVICE_NAME);
    let ws_service = format!("websocket-{}.{}", settings.device_id, SERVICE_NAME);

    assert!(discovered
        .iter()
        .any(|info| info.get_fullname() == grpc_service));
    assert!(discovered
        .iter()
        .any(|info| info.get_fullname() == http_service));
    assert!(discovered
        .iter()
        .any(|info| info.get_fullname() == ws_service));
}
