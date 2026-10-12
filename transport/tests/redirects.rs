//! Loopback-only redirect contract tests; all credentials are synthetic.
use music_player_transport::{HttpReader, HttpRequest, TransportHandle, TransportTerminal};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

const CASE_TIMEOUT: Duration = Duration::from_secs(5);
const SOCKET_TIMEOUT: Duration = Duration::from_secs(2);
const BODY: &[u8] = b"synthetic redirect body";
const HEADERS: [(&str, &str); 4] = [
    ("authorization", "Bearer synthetic-redirect-test"),
    ("cookie", "session=synthetic-redirect-test"),
    ("x-emby-token", "synthetic-emby-token"),
    ("x-caller", "synthetic-custom-header"),
];

fn wait_until(timeout: Duration, mut ready: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    loop {
        if ready() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        thread::sleep(Duration::from_millis(5));
    }
}

fn bounded_join<T>(job: JoinHandle<T>) -> thread::Result<T> {
    if !wait_until(Duration::from_secs(3), || job.is_finished()) {
        // Rust cannot kill a stuck thread. Fail the test process instead of
        // detaching a leaked reader or hanging CI in an unconditional join.
        eprintln!("redirect fixture cleanup watchdog expired");
        std::process::abort();
    }
    job.join()
}

// Deliberately no Debug: assertion failures must not print request headers.
#[derive(Clone)]
struct Seen {
    target: String,
    headers: Vec<(String, String)>,
}

fn read_request(stream: &mut TcpStream, stop: &AtomicBool) -> io::Result<Seen> {
    stream.set_read_timeout(Some(Duration::from_millis(100)))?;
    stream.set_write_timeout(Some(SOCKET_TIMEOUT))?;
    let deadline = Instant::now() + SOCKET_TIMEOUT;
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "fixture read deadline",
            ));
        }
        if bytes.len() >= 16 * 1024 {
            return Err(io::Error::other("fixture request too large"));
        }
        let mut byte = [0];
        match stream.read(&mut byte) {
            Ok(0) => return Err(io::Error::other("fixture incomplete request")),
            Ok(_) => bytes.push(byte[0]),
            Err(e)
                if matches!(
                    e.kind(),
                    io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(e) => return Err(e),
        }
    }
    let text =
        std::str::from_utf8(&bytes).map_err(|_| io::Error::other("fixture request encoding"))?;
    let mut lines = text.split("\r\n");
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    if first.next() != Some("GET") {
        return Err(io::Error::other("fixture expected GET"));
    }
    let target = first
        .next()
        .ok_or_else(|| io::Error::other("fixture missing target"))?
        .to_owned();
    let mut headers = Vec::new();
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| io::Error::other("fixture malformed header"))?;
        headers.push((name.to_ascii_lowercase(), value.trim().to_owned()));
    }
    Ok(Seen { target, headers })
}

enum Reply {
    Redirect(u16, Option<String>),
    Body,
    HoldHeaders,
}

struct Server {
    origin: String,
    stop: Arc<AtomicBool>,
    holding: Arc<AtomicBool>,
    seen: Arc<Mutex<Vec<Seen>>>,
    job: Option<JoinHandle<io::Result<()>>>,
}

impl Server {
    fn start(mut reply: impl FnMut(&Seen, &str) -> Reply + Send + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback fixture");
        listener
            .set_nonblocking(true)
            .expect("nonblocking listener");
        let origin = format!("http://{}", listener.local_addr().expect("fixture address"));
        let stop = Arc::new(AtomicBool::new(false));
        let holding = Arc::new(AtomicBool::new(false));
        let seen = Arc::new(Mutex::new(Vec::new()));
        let (worker_stop, worker_holding, worker_seen, worker_origin) =
            (stop.clone(), holding.clone(), seen.clone(), origin.clone());
        let job = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(20);
            loop {
                if Instant::now() >= deadline {
                    return Err(io::Error::other("fixture lifetime expired"));
                }
                let (mut stream, _) = match listener.accept() {
                    Ok(pair) => pair,
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // Check pending connections before stopping, so an unwanted
                        // target request cannot disappear into the accept backlog.
                        if worker_stop.load(Ordering::Acquire) {
                            return Ok(());
                        }
                        thread::sleep(Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => return Err(e),
                };
                // Accepted sockets use bounded blocking I/O on every platform.
                stream.set_nonblocking(false)?;
                let request = read_request(&mut stream, &worker_stop)?;
                let response = reply(&request, &worker_origin);
                worker_seen.lock().unwrap().push(request);
                match response {
                    Reply::Redirect(status, location) => {
                        let location = location
                            .map(|s| format!("Location: {s}\r\n"))
                            .unwrap_or_default();
                        write!(stream, "HTTP/1.1 {status} Redirect\r\n{location}Content-Length: 0\r\nConnection: close\r\n\r\n")?;
                    }
                    Reply::Body => {
                        write!(
                            stream,
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            BODY.len()
                        )?;
                        stream.write_all(BODY)?;
                    }
                    Reply::HoldHeaders => {
                        worker_holding.store(true, Ordering::Release);
                        while !worker_stop.load(Ordering::Acquire) && Instant::now() < deadline {
                            thread::sleep(Duration::from_millis(5));
                        }
                        // Keep this socket alive until explicit fixture cleanup.
                        drop(stream);
                        worker_holding.store(false, Ordering::Release);
                    }
                }
            }
        });
        Self {
            origin,
            stop,
            holding,
            seen,
            job: Some(job),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.origin)
    }

    fn requests(&self) -> Vec<Seen> {
        self.seen.lock().unwrap().clone()
    }

    fn finish(&mut self) -> io::Result<()> {
        self.stop.store(true, Ordering::Release);
        match self.job.take() {
            Some(job) => bounded_join(job)
                .unwrap_or_else(|_| Err(io::Error::other("fixture thread panicked"))),
            None => Ok(()),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.finish();
    }
}

struct Download {
    handle: TransportHandle,
    job: Option<JoinHandle<io::Result<Vec<u8>>>>,
}

impl Download {
    fn start(request: HttpRequest) -> Self {
        let (mut reader, handle) = HttpReader::start(request).expect("start loopback transport");
        let job = thread::spawn(move || {
            let mut body = Vec::new();
            let result = reader.read_to_end(&mut body).map(|_| body);
            // Completion of this thread proves the owning reader's Drop joined.
            drop(reader);
            result
        });
        Self {
            handle,
            job: Some(job),
        }
    }

    fn finish(&mut self, timeout: Duration) -> io::Result<Vec<u8>> {
        assert!(
            wait_until(timeout, || self.job.as_ref().unwrap().is_finished()),
            "reader completion deadline exceeded"
        );
        let result = bounded_join(self.job.take().unwrap()).expect("reader thread panicked");
        assert!(
            self.handle.snapshot().worker_exited,
            "transport worker was not reclaimed"
        );
        result
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        if let Some(job) = self.job.take() {
            self.handle.cancel();
            let _ = bounded_join(job);
        }
    }
}

fn request(url: String) -> HttpRequest {
    let mut request = HttpRequest::new(url);
    request.connect_timeout = SOCKET_TIMEOUT;
    request.header_timeout = CASE_TIMEOUT;
    request.read_timeout = CASE_TIMEOUT;
    for (name, value) in HEADERS {
        request
            .headers
            .insert(name, value.parse().expect("synthetic header value"));
    }
    request
}

fn assert_headers(request: &Seen) {
    for (name, expected) in HEADERS {
        assert!(
            request
                .headers
                .iter()
                .filter(|(key, _)| key == name)
                .count()
                == 1,
            "caller header missing or duplicated"
        );
        assert!(
            request
                .headers
                .iter()
                .any(|(key, value)| key == name && value == expected),
            "caller header value changed"
        );
    }
}

#[test]
fn same_origin_preserves_headers_for_all_redirect_statuses() {
    let statuses = [301, 302, 303, 307, 308];
    let mut step = 0;
    let mut server = Server::start(move |_, origin| {
        if step == statuses.len() {
            return Reply::Body;
        }
        let status = statuses[step];
        step += 1;
        let location = if step % 2 == 0 {
            format!("{origin}/hop/{step}")
        } else {
            format!("/hop/{step}")
        };
        Reply::Redirect(status, Some(location))
    });
    let request = request(server.url("/start?source=synthetic"));
    assert!(
        request.follow_redirects,
        "redirects must default to enabled"
    );
    let mut download = Download::start(request);
    assert_eq!(download.finish(CASE_TIMEOUT).expect("redirect body"), BODY);
    assert_eq!(download.handle.snapshot().terminal, TransportTerminal::Eof);
    server.finish().expect("fixture cleanup");
    let seen = server.requests();
    assert_eq!(seen.len(), 6);
    assert!(seen[0].target == "/start?source=synthetic");
    for (index, request) in seen.iter().enumerate() {
        assert_headers(request);
        if index > 0 {
            assert!(request.target == format!("/hop/{index}"));
        }
    }
}

#[test]
fn cross_port_preserves_caller_headers_and_exact_signed_query() {
    const TARGET: &str = "/audio?sig=a%2Fb%2Bc%3D&part=1&part=2&blank=&flag&lower=%2f";
    let mut target = Server::start(|_, _| Reply::Body);
    let location = target.url(TARGET);
    let mut source = Server::start(move |_, _| Reply::Redirect(302, Some(location.clone())));
    assert!(source.origin != target.origin);
    let mut download = Download::start(request(
        source.url("/start?api_key=synthetic-original&source_only=yes"),
    ));
    assert_eq!(
        download.finish(CASE_TIMEOUT).expect("cross-port body"),
        BODY
    );
    assert_eq!(download.handle.snapshot().terminal, TransportTerminal::Eof);
    source.finish().expect("source cleanup");
    target.finish().expect("target cleanup");
    let (initial, redirected) = (source.requests(), target.requests());
    assert_eq!(initial.len(), 1);
    assert_eq!(redirected.len(), 1);
    assert_headers(&initial[0]);
    assert_headers(&redirected[0]);
    assert!(
        redirected[0].target == TARGET,
        "signed target query changed"
    );
}

#[test]
fn follow_disabled_returns_302_without_requesting_target() {
    let mut target = Server::start(|_, _| Reply::Body);
    let location = target.url("/must-not-be-requested");
    let mut source = Server::start(move |_, _| Reply::Redirect(302, Some(location.clone())));
    let mut request = request(source.url("/start"));
    request.follow_redirects = false;
    let mut download = Download::start(request);
    assert!(download.finish(CASE_TIMEOUT).is_err());
    assert_eq!(
        download.handle.snapshot().terminal,
        TransportTerminal::HttpStatus(302)
    );
    source.finish().expect("source cleanup");
    target.finish().expect("target cleanup");
    assert_eq!(source.requests().len(), 1);
    assert_eq!(target.requests().len(), 0);
}

#[test]
fn redirect_loop_stops_after_ten_hops_and_eleven_requests() {
    let mut server = Server::start(|_, _| Reply::Redirect(302, Some("/loop".into())));
    let mut download = Download::start(request(server.url("/loop")));
    assert!(download.finish(CASE_TIMEOUT).is_err());
    assert_eq!(
        download.handle.snapshot().terminal,
        TransportTerminal::RedirectLimit
    );
    server.finish().expect("fixture cleanup");
    let seen = server.requests();
    assert_eq!(seen.len(), 11, "initial request plus ten followed hops");
    for request in &seen {
        assert!(request.target == "/loop");
        assert_headers(request);
    }
}

#[test]
fn missing_or_invalid_location_is_invalid_redirect() {
    for location in [
        None,
        Some("http://[invalid"),
        Some("file:///synthetic-media"),
    ] {
        let mut server =
            Server::start(move |_, _| Reply::Redirect(302, location.map(str::to_owned)));
        let mut download = Download::start(request(server.url("/start")));
        assert!(download.finish(CASE_TIMEOUT).is_err());
        assert_eq!(
            download.handle.snapshot().terminal,
            TransportTerminal::InvalidRedirect
        );
        server.finish().expect("fixture cleanup");
        assert_eq!(server.requests().len(), 1);
    }
}

#[test]
fn cancel_after_redirect_reclaims_reader_and_worker_while_target_holds_headers() {
    let mut target = Server::start(|_, _| Reply::HoldHeaders);
    let location = target.url("/delayed-headers");
    let mut source = Server::start(move |_, _| Reply::Redirect(302, Some(location.clone())));
    let mut request = request(source.url("/start"));
    request.header_timeout = Duration::from_secs(30);
    let mut download = Download::start(request);
    assert!(
        wait_until(CASE_TIMEOUT, || target.holding.load(Ordering::Acquire)),
        "redirect target was not reached"
    );
    assert_eq!(
        download.handle.snapshot().terminal,
        TransportTerminal::Running
    );
    assert!(!download.handle.snapshot().worker_exited);
    assert!(!download.job.as_ref().unwrap().is_finished());
    let called = Instant::now();
    download.handle.cancel();
    let error = download
        .finish(Duration::from_secs(2))
        .expect_err("cancel must interrupt read");
    assert!(
        called.elapsed() <= Duration::from_secs(2),
        "cancel exceeded two seconds"
    );
    assert_eq!(error.kind(), io::ErrorKind::ConnectionAborted);
    assert_eq!(
        download.handle.snapshot().terminal,
        TransportTerminal::Cancelled
    );
    // Server cleanup has not run: no response, EOF, or header timeout can explain completion.
    assert!(
        target.holding.load(Ordering::Acquire),
        "fixture released socket before reader Drop"
    );
    assert!(!target.stop.load(Ordering::Acquire));
    source.finish().expect("source cleanup");
    target.finish().expect("target cleanup");
    assert_eq!(source.requests().len(), 1);
    let seen = target.requests();
    assert_eq!(seen.len(), 1);
    assert!(seen[0].target == "/delayed-headers");
    assert_headers(&seen[0]);
}
