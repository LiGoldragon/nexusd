//! cli-msg — the nexus-cli ↔ nexusd protocol.
//!
//! # Decisions (Li 2026-04-25)
//!
//! - **Client-generated request IDs.** Every nexus-cli request
//!   carries a `cli_request_id: u64`. The space is owned by
//!   nexus-cli; nexusd uses it for correlation; criomed never
//!   sees it.
//! - **Indefinite waiting on the client side.** nexus-cli does
//!   *not* time out on its own. It will wait as long as needed
//!   for a reply.
//! - **Heartbeat probe.** nexus-cli periodically sends a
//!   [`CliRequest::Heartbeat`] with the same `cli_request_id`.
//!   nexusd replies with [`CliReply::Working`] or
//!   [`CliReply::Done`].
//! - **Backup-file fallback.** If the requester's socket drops
//!   while nexusd is still working, nexusd writes the eventual
//!   reply to the path the client provided in
//!   [`request::FallbackSpec`]. A later invocation can
//!   [`CliRequest::Resume`] from that path.
//!
//! # Wire layout
//!
//! The wire carries a stream of [`CliFrame`] archives. Both
//! parties know the [`CliFrame`] rkyv schema, so framing is
//! intrinsic to the schema — the archived form encodes its own
//! structure, including any size information needed to delimit
//! one frame from the next. There is no "raw bytes outside
//! rkyv" concept; the universal handshake is the frame type.
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
//! connection model) live in `mentci-next/reports/071`.
//! The types in this module are deliberately policy-free; only
//! the schema commitments are codified here. Policies tune
//! freely as implementation lands.

pub mod fallback;
pub mod frame;
pub mod reply;
pub mod request;

pub use fallback::{FallbackFormat, FallbackSpec};
pub use frame::{CliBody, CliFrame, CliRequestId};
pub use reply::{CliReply, WorkingStage};
pub use request::CliRequest;
