#[cfg(test)]
mod tests;

use async_stream::stream;
use futures_util::Stream;
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use std::thread;

use music_player_settings::{read_settings, Settings};

pub const SERVICE_NAME: &'static str = "_music-player._tcp.local.";
pub const XBMC_SERVICE_NAME: &'static str = "_xbmc-jsonrpc-h._tcp.local.";

pub struct MdnsResponder {
    responder: libmdns::Responder,
    svc: Vec<libmdns::Service>,
}

impl MdnsResponder {
    pub fn new() -> Self {
        let responder = libmdns::Responder::new().unwrap();
        Self {
            responder,
            svc: vec![],
        }
    }

    pub fn register_service(&mut self, name: &str, port: u16) {
        self.svc.push(self.responder.register(
            "_music-player._tcp".to_owned(),
            name.to_owned(),
            port,
            &["path=/"],
        ));
    }
}

pub fn register_services() {
    let config = read_settings().unwrap();
    let settings = config.try_deserialize::<Settings>().unwrap();
    let http_service = format!("http-{}", settings.device_id);
    let ws_service = format!("websocket-{}", settings.device_id);
    let grpc_service = format!("grpc-{}", settings.device_id);

    thread::spawn(move || {
        let mut responder = MdnsResponder::new();
        responder.register_service(&http_service, settings.http_port);
        responder.register_service(&ws_service, settings.ws_port);
        responder.register_service(&grpc_service, settings.port);
        loop {
            ::std::thread::sleep(::std::time::Duration::from_secs(10));
        }
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

pub fn discover(service_name: &str) -> impl Stream<Item = ServiceInfo> {
    let mdns = ServiceDaemon::new().unwrap();
    let receiver = mdns.browse(&service_name).expect("Failed to browse");

    stream! {
        while let Ok(event) = receiver.recv() {
            match event {
                ServiceEvent::ServiceResolved(info) => {
                    yield info;
                }
                _ => {}
            }
        }
    }
}
