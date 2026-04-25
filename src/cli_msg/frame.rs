//! [`CliFrame`] — the wire envelope for nexus-cli ↔ nexusd.
//!
//! Skeleton-as-design. Bodies are `todo!()`. cargo check passes.

use crate::cli_msg::{CliReply, CliRequest};

/// Client-generated correlation ID.
///
/// Every nexus-cli request carries one. nexusd never invents
/// IDs in this space; criomed never sees them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CliRequestId(pub u64);

impl CliRequestId {
    /// Generate a fresh client-side ID. Strategy is the client's
    /// concern — uuid-v7, monotonic counter, hash of pid+seq, etc.
    pub fn fresh() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct CliFrame {
    pub cli_request_id: CliRequestId,
    pub body: CliBody,
}

#[derive(Debug)]
pub enum CliBody {
    Request(CliRequest),
    Reply(CliReply),
}

impl CliFrame {
    /// Encode to length-prefixed rkyv bytes for socket write.
    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }

    /// Decode from length-prefixed rkyv bytes from socket read.
    pub fn decode(_bytes: &[u8]) -> Result<Self, FrameDecodeError> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameDecodeError {
    #[error("malformed length prefix")]
    BadLength,

    #[error("rkyv archive validation failed")]
    BadArchive,

    #[error("trailing bytes after frame")]
    TrailingBytes,
}
