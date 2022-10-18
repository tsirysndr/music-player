use owo_colors::OwoColorize;
use std::{net::IpAddr, thread, time::Duration};

use futures_util::{pin_mut, StreamExt};
use mdns::{Error, Record, RecordKind};
use music_player_settings::{read_settings, Settings};

pub const SERVICE_NAME: &'static str = "_music-player._tcp.local";

pub fn register_services() {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let http_service = format!("http-{}", settings.device_id);
    let ws_service = format!("websocket-{}", settings.device_id);
    let grpc_service = format!("grpc-{}", settings.device_id);

    thread::spawn(move || {
        register(&http_service, settings.http_port);
    });
    thread::spawn(move || {
        register(&grpc_service, settings.port);
    });
    thread::spawn(move || {
        register(&ws_service, settings.ws_port);
    });
}

pub fn register(name: &str, port: u16) {
    /*
    let mut builder = env_logger::Builder::new();
    builder.parse_filters("libmdns=debug");
    builder.init();
    */

    let responder = libmdns::Responder::new().unwrap();
    let _svc = responder.register(
        "_music-player._tcp".to_owned(),
        name.to_owned(),
        port,
        &["path=/"],
    );

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(10));
    }
}

pub async fn discover(service_name: &str) -> Result<(), Error> {
    let stream = mdns::discover::all(service_name, Duration::from_secs(3))?.listen();
    pin_mut!(stream);

    while let Some(Ok(response)) = stream.next().await {
        let host = response.hostname().unwrap_or("unknown");
        let port = response.port().unwrap_or_default();
        let addr = response.records().filter_map(self::to_ip_addr).next();

        if let Some(addr) = addr {
            println!("{} - {}:{}", host.bright_green(), addr, port)
        } else {
            println!("music-player device does not advertise address");
        }
    }
    Ok(())
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
