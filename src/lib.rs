//! nexus — the text-translator daemon library half.
//!
//! Nexus speaks **nexus text** to clients and **signal** (rkyv)
//! to criome. Inside the daemon, parsing converts text → signal
//! frames; rendering converts signal replies → text.
//!
//! The daemon is stateless modulo per-connection state (negotiated
//! protocol version + in-flight subscription registration). There
//! is no correlation-id mapping — replies pair to requests by
//! position on the connection (FIFO).
//!
//! All wire-protocol types live in
//! [signal](https://github.com/LiGoldragon/signal). The
//! parser/encoder primitives live in
//! [nota-codec](https://github.com/LiGoldragon/nota-codec).
//! This library holds nexus-daemon-specific helpers (errors,
//! soon: connection-state types, request-routing actor).

pub mod error;

pub use error::{Error, Result};
