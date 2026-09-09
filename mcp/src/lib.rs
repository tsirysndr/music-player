//! A Model Context Protocol server for music-player.
//!
//! It turns the daemon into something an agent can DJ with: ask what is
//! playing, search the library, build a set, and run the transport. The agent
//! is a *client of the daemon*, not a second player — everything goes over the
//! same gRPC API the desktop, the TUI and the web client use, so a queue an
//! agent builds is the queue every one of them shows.
//!
//! The transport is stdio: the agent's host spawns `music-player mcp` and
//! speaks newline-delimited JSON-RPC 2.0 over the pipe. That is why nothing in
//! this crate may print to stdout — stdout *is* the protocol. Diagnostics go to
//! stderr through `tracing`, which the host shows in its logs.

mod protocol;
mod session;
mod tools;

pub use protocol::serve;
pub use session::Session;
