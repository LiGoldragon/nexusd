//! nexusd — the nexus-text messenger.
//!
//! Parses nexus messages over a Unix socket (or stdio), forwards
//! rkyv criome-messages to criomed, relays replies back as nexus
//! text. Stateless modulo in-flight request correlations — criomed
//! (sema's engine) and lojixd (owner of lojix-store) hold the state.

mod error;

use error::Result;

fn main() -> Result<()> {
    Ok(())
}
