//! nexusd — library half.
//!
//! Two protocol layers sit at nexusd's boundaries:
//!
//! - [`client_msg`] — the protocol any client (nexus-cli, editor
//!   LSP, scripts, agents, future tools) speaks to nexusd. Carries
//!   nexus text + a *client-generated* request ID. Heartbeats,
//!   cancel, resume, fallback-file delivery for disconnected
//!   requesters. Criomed never sees this layer.
//! - `criome_msg` — the nexusd ↔ criomed protocol (rkyv envelopes
//!   carrying parsed nexus operations). Designed in
//!   `mentci-next/reports/070` §6. Will live in its own crate
//!   `criome-msg` (CANON-MISSING) once that crate is created.
//!
//! nexusd is stateless modulo in-flight request correlations on
//! both sides. `client_msg` correlates by client-generated
//! `RequestId`; `criome-msg` correlates by `correlation_id`. The
//! two ID spaces are independent — nexusd maintains the mapping
//! internally.
//!
//! See `mentci-next/reports/070` and `reports/071` for the full
//! nexus language design and contract / protocol sketches.

pub mod client_msg;
pub mod error;
