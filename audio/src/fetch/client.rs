use anyhow::Error;
use futures_util::{future::IntoStream, FutureExt};
use hyper::{
    client::{self, HttpConnector, ResponseFuture},
    header::RANGE,
    Request,
};
use hyper_rustls::{ConfigBuilderExt, HttpsConnector};

pub struct Client {
    client: hyper::Client<HttpsConnector<HttpConnector>>,
}

impl Client {
    pub fn new() -> Self {
        let tls = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_native_roots()
            .with_no_client_auth();
        // Prepare the HTTPS connector
        let https = hyper_rustls::HttpsConnectorBuilder::new()
            .with_tls_config(tls)
            .https_or_http()
            .enable_http1()
            .build();
        let client = client::Client::builder().build(https);
        Self { client }
    }

    pub fn stream_from_url(
        &self,
        url: &str,
        offset: usize,
        length: usize,
    ) -> Result<IntoStream<ResponseFuture>, Error> {
        let req = Request::builder()
            .method("GET")
            .uri(url)
            .header(RANGE, format!("bytes={}-{}", offset, offset + length - 1))
            .body(hyper::Body::empty())?;
        Ok(self.client.request(req).into_stream())
    }
}
