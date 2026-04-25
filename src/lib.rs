//! nexusd — library half.
//!
//! Two protocol layers sit at nexusd's boundaries:
//!
//! - [`cli_msg`] — the nexus-cli ↔ nexusd protocol. Carries nexus
//!   text + a *client-generated* request ID. Heartbeats, cancel,
//!   resume, fallback-file delivery for disconnected requesters.
//!   Criomed never sees this layer.
//! - `criome_msg` — the nexusd ↔ criomed protocol (rkyv envelopes
//!   carrying parsed nexus operations). Designed in
//!   `mentci-next/reports/070` §6. Will live in its own crate
//!   `criome-msg` (CANON-MISSING) once that crate is created.
//!
//! nexusd is stateless modulo in-flight request correlations on
//! both sides. `cli_msg` correlates by `cli_request_id`;
//! `criome-msg` correlates by `correlation_id`. The two ID spaces
//! are independent — nexusd maintains the mapping internally.
//!
//! See `mentci-next/reports/070` for the full nexus language
//! design and contract sketches.

pub mod cli_msg;
pub mod error;
