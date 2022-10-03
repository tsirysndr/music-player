use futures_util::{
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use music_player_settings::{read_settings, Settings};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

pub struct WebsocketClient {
    pub write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
    pub read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl WebsocketClient {
    pub async fn new() -> Self {
        let config = read_settings().unwrap();
        let settings = config.try_deserialize::<Settings>().unwrap();
        let url = format!("ws://[::1]:{}", settings.ws_port);
        let url = Url::parse(&url).unwrap();
        let (ws_stream, _) = connect_async(url)
            .await
            .expect("Failed to connect to websocket server");
        let (write, read) = ws_stream.split();
        Self { write, read }
    }
}
