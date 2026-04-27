//! nexus — the nexus-text translator daemon.
//!
//! Binds the user-facing UDS at `$NEXUS_SOCKET` (default
//! `/tmp/nexus.sock`); each accepted connection opens its own
//! paired criome connection at `$CRIOME_SOCKET` (default
//! `/tmp/criome.sock`) and runs a one-shot text shuttle until
//! the client closes the write side.

use std::path::PathBuf;

use nexus::{Daemon, Result};

const DEFAULT_NEXUS_SOCKET: &str = "/tmp/nexus.sock";
const DEFAULT_CRIOME_SOCKET: &str = "/tmp/criome.sock";

#[tokio::main]
async fn main() -> Result<()> {
    let listen_path: PathBuf = std::env::var("NEXUS_SOCKET")
        .unwrap_or_else(|_| DEFAULT_NEXUS_SOCKET.to_string())
        .into();
    let criome_socket_path: PathBuf = std::env::var("CRIOME_SOCKET")
        .unwrap_or_else(|_| DEFAULT_CRIOME_SOCKET.to_string())
        .into();

    eprintln!("nexus-daemon: forwarding to criome at {}", criome_socket_path.display());
    eprintln!("nexus-daemon: binding UDS at {}", listen_path.display());
    let daemon = Daemon::new(listen_path, criome_socket_path);

    eprintln!("nexus-daemon: ready");
    daemon.run().await
}
