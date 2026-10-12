//! Output-backend selection for [`Player`](crate::Player).
//!
//! The engine thread always produces the same thing — interleaved **stereo
//! `i16` at the output rate** — into a ring buffer. [`OutputConfig`] chooses
//! who drains that ring and where the samples go:
//!
//! | Backend                | What it does                                              |
//! | ---------------------- | -------------------------------------------------------- |
//! | [`OutputConfig::Cpal`] | System audio device via `cpal` (default; needs `cpal`).  |
//! | `Stdout`               | Raw **S16LE** stereo on stdout — pipe to a player.        |
//! | `Fifo`                 | Raw S16LE into a named FIFO (e.g. a Snapcast pipe).       |
//! | `Unix`                 | Raw S16LE over a Unix-domain socket (listen or connect). |
//! | `Tcp`                  | Raw S16LE over TCP (listen or connect).                  |
//!
//! Every non-`cpal` backend emits the exact same byte stream, real-time
//! paced, so it drops straight into a terminal player:
//!
//! ```sh
//! # stdout → ffplay
//! my-app --output stdout song.flac | ffplay -f s16le -ar 44100 -ac 2 -
//! ```
//!
//! **Important:** in `Stdout` mode fd 1 carries the PCM stream, so the host
//! program must keep stdout clean — send *all* logs/diagnostics to stderr.
//!
//! Backends parse from a compact string via [`str::parse`], which the FFI
//! layer and the `play` example use:
//!
//! ```text
//! cpal
//! stdout            (or "-")
//! fifo:/tmp/snapfifo
//! unix:/tmp/rb.sock            (listen)   unix-connect:/tmp/rb.sock
//! tcp:127.0.0.1:9000           (listen)   tcp-connect:192.168.1.9:9000
//! ```

use std::io::{self, Write};
use std::path::PathBuf;

/// Whether a socket backend binds and accepts clients, or dials out to a
/// receiver that is already listening.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketMode {
    /// Bind the address/path and stream to whoever connects. This is the
    /// default for `tcp:` / `unix:` — a player connects *in* (e.g.
    /// `ffplay tcp://127.0.0.1:9000`).
    Listen,
    /// Dial out to an address/path that is already listening (e.g. a
    /// Snapcast TCP source). The receiver must be up at construction time.
    Connect,
}

/// Where [`Player`](crate::Player) sends its audio. See the [module
/// docs](self) for the string syntax and the byte format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputConfig {
    /// System audio device via `cpal`. Requires the `cpal` feature (on by
    /// default). This is the [`Default`].
    Cpal,
    /// Raw interleaved **S16LE** stereo on stdout (fd 1). Pipe to any
    /// player that reads raw PCM. The host program must not write anything
    /// else to stdout.
    Stdout,
    /// Raw S16LE stereo into a named FIFO at this path. The FIFO must
    /// already exist (`mkfifo`); it is opened read+write so a permanent
    /// writer reference is held and readers never see EOF between tracks.
    Fifo(PathBuf),
    /// Raw S16LE stereo over a Unix-domain socket.
    Unix {
        /// Filesystem path of the socket.
        path: PathBuf,
        /// Listen (bind + accept) or connect (dial out).
        mode: SocketMode,
    },
    /// Raw S16LE stereo over TCP.
    Tcp {
        /// `host:port` (or `:port` to bind all interfaces when listening).
        addr: String,
        /// Listen (bind + accept) or connect (dial out).
        mode: SocketMode,
    },
}

impl OutputConfig {
    /// Open the byte-stream writer for a non-`cpal` backend. The `Cpal`
    /// variant has no writer (it is device-driven) and returns an error.
    ///
    /// For a *listening* socket ([`SocketMode::Listen`]) this **blocks**
    /// until a client connects; for a *connecting* one the receiver must
    /// already be up.
    pub(crate) fn open_writer(&self) -> io::Result<Box<dyn Write + Send>> {
        match self {
            OutputConfig::Cpal => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "cpal backend has no byte-stream writer",
            )),
            // fd 1. The host program must keep stdout otherwise clean —
            // logs go to stderr — so the raw PCM is pipeable to a player.
            OutputConfig::Stdout => Ok(Box::new(io::stdout())),
            OutputConfig::Fifo(path) => open_fifo(path),
            OutputConfig::Unix { path, mode } => open_unix(path, *mode),
            OutputConfig::Tcp { addr, mode } => open_tcp(addr, *mode),
        }
    }
}

/// Open a named FIFO for writing. Opened **read+write** so a permanent
/// writer reference is held — a reader (Snapcast) never sees EOF between
/// tracks. The FIFO must already exist (`mkfifo`).
#[cfg(unix)]
fn open_fifo(path: &std::path::Path) -> io::Result<Box<dyn Write + Send>> {
    use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
    let f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .custom_flags(libc::O_NONBLOCK)
        .open(path)?;
    if !f.metadata()?.file_type().is_fifo() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "output path is not a FIFO",
        ));
    }
    Ok(Box::new(f))
}

#[cfg(not(unix))]
fn open_fifo(_path: &std::path::Path) -> io::Result<Box<dyn Write + Send>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "FIFO output is only supported on Unix",
    ))
}

#[cfg(unix)]
fn open_unix(path: &std::path::Path, mode: SocketMode) -> io::Result<Box<dyn Write + Send>> {
    use std::os::unix::net::{UnixListener, UnixStream};
    match mode {
        SocketMode::Listen => {
            // Remove a stale socket file so bind() doesn't fail with EADDRINUSE.
            let _ = std::fs::remove_file(path);
            let listener = UnixListener::bind(path)?;
            let (stream, _) = listener.accept()?;
            stream.set_nonblocking(true)?;
            Ok(Box::new(stream))
        }
        SocketMode::Connect => {
            let stream = UnixStream::connect(path)?;
            stream.set_nonblocking(true)?;
            Ok(Box::new(stream))
        }
    }
}

#[cfg(not(unix))]
fn open_unix(_path: &std::path::Path, _mode: SocketMode) -> io::Result<Box<dyn Write + Send>> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Unix-socket output is only supported on Unix",
    ))
}

fn open_tcp(addr: &str, mode: SocketMode) -> io::Result<Box<dyn Write + Send>> {
    use std::net::{TcpListener, TcpStream};
    let stream = match mode {
        SocketMode::Listen => {
            let listener = TcpListener::bind(addr)?;
            listener.accept()?.0
        }
        SocketMode::Connect => TcpStream::connect(addr)?,
    };
    // Latency over throughput: PCM chunks are small and time-sensitive.
    stream.set_nonblocking(true)?;
    stream.set_nodelay(true).ok();
    Ok(Box::new(stream))
}

impl Default for OutputConfig {
    // Not derivable: the default is feature-dependent (with only one `cfg`
    // arm compiled in, clippy sees a constant and thinks it could be
    // `#[derive(Default)]`, but the chosen variant differs per feature).
    #[allow(clippy::derivable_impls)]
    fn default() -> Self {
        // When `cpal` is compiled out there is no device to open, so the
        // sensible zero-config default becomes the raw stdout stream.
        #[cfg(feature = "cpal")]
        {
            OutputConfig::Cpal
        }
        #[cfg(not(feature = "cpal"))]
        {
            OutputConfig::Stdout
        }
    }
}

/// Error returned when an [`OutputConfig`] string cannot be parsed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseOutputError(pub String);

impl std::fmt::Display for ParseOutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid audio output spec: {}", self.0)
    }
}

impl std::error::Error for ParseOutputError {}

impl std::str::FromStr for OutputConfig {
    type Err = ParseOutputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        // `scheme:rest` where the scheme is everything up to the first ':'.
        // stdout/cpal have no argument.
        let (scheme, rest) = match s.split_once(':') {
            Some((a, b)) => (a, Some(b)),
            None => (s, None),
        };
        let arg = |rest: Option<&str>| -> Result<String, ParseOutputError> {
            match rest {
                Some(r) if !r.is_empty() => Ok(r.to_string()),
                _ => Err(ParseOutputError(format!("`{scheme}` needs an argument"))),
            }
        };
        match scheme {
            "cpal" => Ok(OutputConfig::Cpal),
            "stdout" | "-" => Ok(OutputConfig::Stdout),
            "fifo" => Ok(OutputConfig::Fifo(PathBuf::from(arg(rest)?))),
            "unix" | "unix-listen" => Ok(OutputConfig::Unix {
                path: PathBuf::from(arg(rest)?),
                mode: SocketMode::Listen,
            }),
            "unix-connect" => Ok(OutputConfig::Unix {
                path: PathBuf::from(arg(rest)?),
                mode: SocketMode::Connect,
            }),
            "tcp" | "tcp-listen" => Ok(OutputConfig::Tcp {
                addr: arg(rest)?,
                mode: SocketMode::Listen,
            }),
            "tcp-connect" => Ok(OutputConfig::Tcp {
                addr: arg(rest)?,
                mode: SocketMode::Connect,
            }),
            other => Err(ParseOutputError(format!("unknown backend `{other}`"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_backend() {
        assert_eq!("cpal".parse::<OutputConfig>().unwrap(), OutputConfig::Cpal);
        assert_eq!("-".parse::<OutputConfig>().unwrap(), OutputConfig::Stdout);
        assert_eq!(
            "stdout".parse::<OutputConfig>().unwrap(),
            OutputConfig::Stdout
        );
        assert_eq!(
            "fifo:/tmp/snapfifo".parse::<OutputConfig>().unwrap(),
            OutputConfig::Fifo(PathBuf::from("/tmp/snapfifo"))
        );
        assert_eq!(
            "unix:/tmp/rb.sock".parse::<OutputConfig>().unwrap(),
            OutputConfig::Unix {
                path: PathBuf::from("/tmp/rb.sock"),
                mode: SocketMode::Listen,
            }
        );
        assert_eq!(
            "unix-connect:/tmp/rb.sock".parse::<OutputConfig>().unwrap(),
            OutputConfig::Unix {
                path: PathBuf::from("/tmp/rb.sock"),
                mode: SocketMode::Connect,
            }
        );
        assert_eq!(
            "tcp:127.0.0.1:9000".parse::<OutputConfig>().unwrap(),
            OutputConfig::Tcp {
                addr: "127.0.0.1:9000".to_string(),
                mode: SocketMode::Listen,
            }
        );
        assert_eq!(
            "tcp-connect:192.168.1.9:9000"
                .parse::<OutputConfig>()
                .unwrap(),
            OutputConfig::Tcp {
                addr: "192.168.1.9:9000".to_string(),
                mode: SocketMode::Connect,
            }
        );
    }

    #[test]
    fn rejects_garbage_and_missing_args() {
        assert!("bogus".parse::<OutputConfig>().is_err());
        assert!("fifo".parse::<OutputConfig>().is_err());
        assert!("tcp:".parse::<OutputConfig>().is_err());
    }
}
