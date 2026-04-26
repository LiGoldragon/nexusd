//! nexus — the nexus-text translator daemon.
//!
//! Listens on a Unix socket, parses incoming nexus text into
//! signal frames, forwards to criome, renders replies back as
//! nexus text. Stateless modulo per-connection state.

use nexus::error::Result;

fn main() -> Result<()> {
    Ok(())
}
