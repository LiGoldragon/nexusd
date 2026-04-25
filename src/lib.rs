//! nexus — library half.
//!
//! Two protocol layers sit at nexus's boundaries:
//!
//! - [`client_msg`] — the protocol any client (nexus-cli, editor
//!   LSP, scripts, agents, future tools) speaks to nexus. Carries
//!   nexus text + a *client-generated* request ID. Heartbeats,
//!   cancel, resume, fallback-file delivery for disconnected
//!   requesters. Criomed never sees this layer.
//! - `criome_msg` — the nexus ↔ criome protocol (rkyv envelopes
//!   carrying parsed nexus operations). Designed in
//!   `mentci/reports/070` §6. Will live in its own crate
//!   `signal` (CANON-MISSING) once that crate is created.
//!
//! nexus is stateless modulo in-flight request correlations on
//! both sides. `client_msg` correlates by client-generated
//! `RequestId`; `signal` correlates by `correlation_id`. The
//! two ID spaces are independent — nexus maintains the mapping
//! internally.
//!
//! See `mentci/reports/070` and `reports/071` for the full
//! nexus language design and contract / protocol sketches.

pub mod client_msg;
pub mod error;
