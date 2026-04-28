//! nexus-daemon — entry point.
//!
//! Reads `$NEXUS_SOCKET` (default `/tmp/nexus.sock`) and
//! `$CRIOME_SOCKET` (default `/tmp/criome.sock`) from the
//! environment, brings up the [`Daemon`] supervision tree
//! ([`Listener`](nexus::listener::Listener) + per-client
//! [`Connection`](nexus::connection::Connection) actors),
//! waits.

use std::path::PathBuf;

use ractor::Actor;

use nexus::daemon::{Arguments, Daemon};
use nexus::{Error, Result};

const DEFAULT_NEXUS_SOCKET: &str = "/tmp/nexus.sock";
const DEFAULT_CRIOME_SOCKET: &str = "/tmp/criome.sock";

#[tokio::main]
async fn main() -> Result<()> {
    let socket_path: PathBuf = std::env::var("NEXUS_SOCKET")
        .unwrap_or_else(|_| DEFAULT_NEXUS_SOCKET.to_string())
        .into();
    let criome_socket_path: PathBuf = std::env::var("CRIOME_SOCKET")
        .unwrap_or_else(|_| DEFAULT_CRIOME_SOCKET.to_string())
        .into();

    eprintln!("nexus-daemon: forwarding to criome at {}", criome_socket_path.display());
    eprintln!("nexus-daemon: binding UDS at {}", socket_path.display());

    let (_daemon_ref, daemon_handle) = Actor::spawn(
        Some("daemon".into()),
        Daemon,
        Arguments { socket_path, criome_socket_path },
    )
    .await
    .map_err(|error| Error::ActorSpawn(error.to_string()))?;

    eprintln!("nexus-daemon: ready");
    daemon_handle
        .await
        .map_err(|error| Error::ActorCall(format!("daemon join: {error}")))?;
    Ok(())
}
