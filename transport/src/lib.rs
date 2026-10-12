//! Cancellable HTTP body -> bounded synchronous reader bridge.
//! No media files, retries, or whole-response timeout. Redirects are followed
//! by default, retaining caller-supplied headers across the chain.
use std::io::{self, Read};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::{mpsc, watch};

pub struct HttpRequest {
    pub url: String,
    pub headers: reqwest::header::HeaderMap,
    /// Follow HTTP redirects, retaining request headers including credentials.
    /// Defaults to true; false returns the original redirect status unchanged.
    /// At most ten hops share the single header timeout and cancellation scope.
    pub follow_redirects: bool,
    pub connect_timeout: Duration,
    pub header_timeout: Duration,
    pub read_timeout: Duration,
}
impl HttpRequest {
    pub fn new(url: String) -> Self {
        Self {
            url,
            headers: Default::default(),
            follow_redirects: true,
            connect_timeout: Duration::from_secs(10),
            header_timeout: Duration::from_secs(15),
            read_timeout: Duration::from_secs(30),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportTerminal {
    Running,
    Eof,
    Cancelled,
    HttpStatus(u16),
    RedirectLimit,
    InvalidRedirect,
    HeaderTimeout,
    ReadTimeout,
    NetworkError,
}
#[derive(Debug, Clone)]
pub struct TransportSnapshot {
    pub terminal: TransportTerminal,
    pub worker_exited: bool,
}
struct Shared {
    cancel: watch::Sender<bool>,
    terminal: Mutex<TransportTerminal>,
    worker_exited: AtomicBool,
}
#[derive(Clone)]
pub struct TransportHandle(Arc<Shared>);
impl TransportHandle {
    pub fn cancel(&self) {
        self.0.cancel.send_replace(true);
    }
    pub fn snapshot(&self) -> TransportSnapshot {
        TransportSnapshot {
            terminal: self.0.terminal.lock().unwrap().clone(),
            worker_exited: self.0.worker_exited.load(Ordering::Acquire),
        }
    }
    fn cancelled(&self) -> bool {
        *self.0.cancel.borrow()
    }
}
/// Owns the transport worker. Drop cancels and joins; no detached task.
pub struct HttpReader {
    rx: mpsc::Receiver<Vec<u8>>,
    current: Vec<u8>,
    position: usize,
    handle: TransportHandle,
    worker: Option<JoinHandle<()>>,
}
impl HttpReader {
    pub fn start(request: HttpRequest) -> io::Result<(Self, TransportHandle)> {
        let url = reqwest::Url::parse(&request.url).map_err(|_| invalid())?;
        if !is_media_url(&url)
            || request.connect_timeout.is_zero()
            || request.header_timeout.is_zero()
            || request.read_timeout.is_zero()
        {
            return Err(invalid());
        }
        let (cancel, cancel_rx) = watch::channel(false);
        let handle = TransportHandle(Arc::new(Shared {
            cancel,
            terminal: Mutex::new(TransportTerminal::Running),
            worker_exited: AtomicBool::new(false),
        }));
        let (tx, rx) = mpsc::channel(4);
        let owner = handle.clone();
        let worker = std::thread::Builder::new()
            .name("media-http".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();
                let terminal = match runtime {
                    Ok(rt) => {
                        let result = rt.block_on(download(request, url, &tx, cancel_rx));
                        drop(rt);
                        result
                    }
                    Err(_) => TransportTerminal::NetworkError,
                };
                *owner.0.terminal.lock().unwrap() = terminal;
                // Publish terminal before disconnecting the blocking receiver.
                drop(tx);
                owner.0.worker_exited.store(true, Ordering::Release);
            })
            .map_err(|_| io::Error::other("cannot start media transport"))?;
        Ok((
            Self {
                rx,
                current: Vec::new(),
                position: 0,
                handle: handle.clone(),
                worker: Some(worker),
            },
            handle,
        ))
    }
}
fn is_media_url(url: &reqwest::Url) -> bool {
    matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none()
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, "invalid media HTTP request")
}
async fn cancelled(rx: &mut watch::Receiver<bool>) {
    loop {
        if *rx.borrow_and_update() {
            return;
        }
        if rx.changed().await.is_err() {
            return;
        }
    }
}
async fn open(
    request: &HttpRequest,
    client: &reqwest::Client,
    mut url: reqwest::Url,
) -> Result<reqwest::Response, TransportTerminal> {
    let mut hops = 0;
    loop {
        let mut headers = request.headers.clone();
        if hops > 0 {
            // Derive Host from the new URL; all end-to-end headers, including
            // credentials, keep their original values across origins.
            headers.remove(reqwest::header::HOST);
        }
        let response = client
            .get(url.clone())
            .headers(headers)
            .send()
            .await
            .map_err(|_| TransportTerminal::NetworkError)?;
        if !request.follow_redirects
            || !matches!(response.status().as_u16(), 301 | 302 | 303 | 307 | 308)
        {
            return Ok(response);
        }
        if hops >= 10 {
            return Err(TransportTerminal::RedirectLimit);
        }
        let location = response
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|value| value.to_str().ok())
            .filter(|value| !value.is_empty())
            .ok_or(TransportTerminal::InvalidRedirect)?;
        let next = url
            .join(location)
            .map_err(|_| TransportTerminal::InvalidRedirect)?;
        if !is_media_url(&next) {
            return Err(TransportTerminal::InvalidRedirect);
        }
        // Only the final response is audio. Location carries its own query;
        // do not append original query parameters to a signed destination.
        drop(response);
        url = next;
        hops += 1;
    }
}
async fn download(
    request: HttpRequest,
    url: reqwest::Url,
    tx: &mpsc::Sender<Vec<u8>>,
    mut cancel: watch::Receiver<bool>,
) -> TransportTerminal {
    let client = match reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(request.connect_timeout)
        .hickory_dns(true)
        .build()
    {
        Ok(c) => c,
        Err(_) => return TransportTerminal::NetworkError,
    };
    let mut response = tokio::select! {
        biased;
        _=cancelled(&mut cancel)=>return TransportTerminal::Cancelled,
        // The deadline includes all redirects, not a fresh timeout per hop.
        result=tokio::time::timeout(request.header_timeout,open(&request, &client, url))=>match result {
            Err(_)=>return TransportTerminal::HeaderTimeout,
            Ok(Err(terminal))=>return terminal,
            Ok(Ok(r))=>r,
        }
    };
    if response.status() != reqwest::StatusCode::OK {
        return TransportTerminal::HttpStatus(response.status().as_u16());
    }
    loop {
        let chunk = tokio::select! {
            biased;
            _=cancelled(&mut cancel)=>return TransportTerminal::Cancelled,
            result=tokio::time::timeout(request.read_timeout,response.chunk())=>match result {
                Err(_)=>return TransportTerminal::ReadTimeout,
                Ok(Err(_))=>return TransportTerminal::NetworkError,
                Ok(Ok(None))=>return TransportTerminal::Eof,
                Ok(Ok(Some(b)))=>b,
            }
        };
        // Bound retained response frames as well as the bridge queue.
        if chunk.len() > 1024 * 1024 {
            return TransportTerminal::NetworkError;
        }
        for part in chunk.chunks(32 * 1024) {
            tokio::select! {
                biased;
                _=cancelled(&mut cancel)=>return TransportTerminal::Cancelled,
                result=tx.send(part.to_vec())=>if result.is_err(){return TransportTerminal::Cancelled;}
            }
        }
    }
}
impl Read for HttpReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        if self.handle.cancelled() {
            return Err(cancel_error());
        }
        if self.position == self.current.len() {
            self.current.clear();
            self.position = 0;
            match self.rx.blocking_recv() {
                Some(bytes) => self.current = bytes,
                None => {
                    if self.handle.cancelled() {
                        return Err(cancel_error());
                    }
                    return match self.handle.snapshot().terminal {
                        TransportTerminal::Eof => Ok(0),
                        TransportTerminal::Cancelled => Err(cancel_error()),
                        TransportTerminal::HeaderTimeout | TransportTerminal::ReadTimeout => Err(
                            io::Error::new(io::ErrorKind::TimedOut, "media transport stalled"),
                        ),
                        TransportTerminal::RedirectLimit => {
                            Err(io::Error::other("media redirect limit exceeded"))
                        }
                        TransportTerminal::InvalidRedirect => Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "invalid media redirect",
                        )),
                        TransportTerminal::HttpStatus(_) => Err(io::Error::new(
                            io::ErrorKind::PermissionDenied,
                            "media HTTP status rejected",
                        )),
                        _ => Err(io::Error::other("media transport failed")),
                    };
                }
            }
        }
        if self.handle.cancelled() {
            return Err(cancel_error());
        }
        let n = buf.len().min(self.current.len() - self.position);
        buf[..n].copy_from_slice(&self.current[self.position..self.position + n]);
        self.position += n;
        Ok(n)
    }
}
fn cancel_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::ConnectionAborted,
        "media transport cancelled",
    )
}
impl Drop for HttpReader {
    fn drop(&mut self) {
        self.handle.cancel();
        self.rx.close();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
