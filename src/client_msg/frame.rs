//! [`Frame`] — the wire envelope for any client ↔ nexusd.
//!
//! Skeleton-as-design. Bodies are `todo!()`. cargo check passes.

use crate::client_msg::{Reply, Request};

/// Client-generated correlation ID.
///
/// Every client request carries one. nexusd never invents IDs
/// in this space; criomed never sees them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RequestId(pub u128);

impl RequestId {
    /// Generate a fresh client-side ID. Strategy is the client's
    /// concern — uuid-v7 recommended; monotonic counter
    /// acceptable for processes that don't share a fallback
    /// directory.
    pub fn fresh() -> Self {
        todo!()
    }
}

#[derive(Debug)]
pub struct Frame {
    pub request_id: RequestId,
    pub body: Body,
}

#[derive(Debug)]
pub enum Body {
    Request(Request),
    Reply(Reply),
}

impl Frame {
    /// Encode to rkyv-archive bytes for socket write.
    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }

    /// Decode from rkyv-archive bytes off the socket.
    pub fn decode(_bytes: &[u8]) -> Result<Self, FrameDecodeError> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FrameDecodeError {
    #[error("rkyv archive validation failed")]
    BadArchive,

    #[error("trailing bytes after frame")]
    TrailingBytes,
}
