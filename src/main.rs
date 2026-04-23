//! nexusd — the sema database daemon.
//!
//! Accepts nexus messages over a Unix socket (or stdio), parses via
//! nexus-serde, dispatches to sema operations, responds in nexus.
//! Ractor-hosted services own the daemon's concurrent state.

fn main() -> anyhow::Result<()> {
    Ok(())
}
