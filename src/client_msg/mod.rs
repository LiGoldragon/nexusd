//! client_msg — the protocol any client speaks to nexusd.
//!
//! Clients include `nexus-cli`, editor LSPs, scripts, agents,
//! and any future tool that needs to author nexus messages and
//! receive replies. They all share this protocol.
//!
//! # Decisions (Li 2026-04-25)
//!
//! - **Client-generated request IDs.** Every client request
//!   carries a [`frame::RequestId`]. The space is the client's;
//!   nexusd uses it for correlation; criome never sees it.
//! - **Indefinite waiting on the client side.** Clients do *not*
//!   time out on their own. They wait as long as needed for a
//!   reply.
//! - **Heartbeat probe.** Clients periodically send a
//!   [`Request::Heartbeat`] with the same [`frame::RequestId`].
//!   nexusd replies with [`Reply::Working`] or [`Reply::Done`].
//! - **Backup-file fallback.** If the requester's socket drops
//!   while nexusd is still working, nexusd writes the eventual
//!   reply to the path the client provided in
//!   [`fallback::FallbackSpec`]. A later [`Request::Resume`]
//!   can pick the reply up.
//!
//! # Wire layout
//!
//! The wire carries a stream of [`Frame`] archives. Both parties
//! know the [`Frame`] rkyv schema, so framing is intrinsic to the
//! schema — the universal handshake is the frame type itself.
//! There is no "raw bytes outside rkyv" concept.
//!
//! The same socket path serves all clients; per-connection
//! isolation is the OS's job. No upper bound on frame size at
//! the protocol level (decoders MAY refuse > a configured
//! threshold).
//!
//! # Policy reference
//!
//! Detailed policies (heartbeat interval, fallback file path,
//! request-id strategy, nexusd state machine, cancel semantics,
//! connection model) live in `mentci-next/reports/071`. The
//! types in this module are deliberately policy-free; only the
//! schema commitments are codified here. Policies tune freely
//! as implementation lands.

pub mod fallback;
pub mod frame;
pub mod path;
pub mod reply;
pub mod request;

pub use fallback::{FallbackFormat, FallbackSpec};
pub use frame::{Body, Frame, RequestId};
pub use path::WirePath;
pub use reply::{Reply, WorkingStage};
pub use request::Request;
