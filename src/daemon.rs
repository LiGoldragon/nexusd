//! `Daemon` — the nexus translator process.
//!
//! Owns the two configured socket paths (the user-facing UDS to
//! bind on; the criome UDS to dial out to per connection). The
//! daemon itself holds no per-connection state — that lives on
//! [`Connection`](crate::connection::Connection) and dies with
//! the connection — so the type's only field is configuration.
//!
//! The daemon is unlike criome's [`Daemon`](https://github.com/LiGoldragon/criome/blob/main/src/daemon.rs)
//! in that it has no shared per-process state to share via
//! `Arc`; each accepted connection opens its own paired
//! [`CriomeLink`](crate::criome_link::CriomeLink) when it sees
//! its first request.

use std::path::PathBuf;

use tokio::net::UnixListener;

use crate::connection::Connection;
use crate::error::Result;

pub struct Daemon {
    listen_path: PathBuf,
    criome_socket_path: PathBuf,
}

impl Daemon {
    pub fn new(listen_path: PathBuf, criome_socket_path: PathBuf) -> Self {
        Self { listen_path, criome_socket_path }
    }

    /// Bind the user-facing UDS, then accept connections forever
    /// — each connection runs its own text shuttle on a tokio
    /// task. Removes any stale socket file first.
    pub async fn run(self) -> Result<()> {
        let _ = std::fs::remove_file(&self.listen_path);
        let listener = UnixListener::bind(&self.listen_path)?;
        loop {
            let (client, _) = listener.accept().await?;
            let criome_socket_path = self.criome_socket_path.clone();
            tokio::spawn(async move {
                let connection = Connection::new(client, criome_socket_path);
                if let Err(error) = connection.shuttle().await {
                    eprintln!("nexus: connection error: {error}");
                }
            });
        }
    }
}
