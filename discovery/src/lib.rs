use mdns_sd::{ServiceDaemon, ServiceEvent};
use owo_colors::OwoColorize;
use std::{net::IpAddr, thread, time::Duration};

use futures_util::{pin_mut, StreamExt};
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

pub fn discover(service_name: &str) {
    let mdns = ServiceDaemon::new().unwrap();
    let service_name = format!("{}.", service_name);
    let receiver = mdns.browse(&service_name).expect("Failed to browse");
    while let Ok(event) = receiver.recv() {
        match event {
            ServiceEvent::ServiceResolved(info) => {
                println!(
                    "{} - {} - {:?} - port: {}",
                    info.get_fullname().bright_green(),
                    info.get_hostname().to_lowercase(),
                    info.get_addresses(),
                    info.get_port()
                );
            }
            _ => {}
        }
    }
}
