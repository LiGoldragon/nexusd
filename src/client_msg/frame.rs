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
    /// Generate a fresh client-side ID using UUID v7 (time-
    /// ordered, globally unique). Per
    /// `mentci-next/reports/071` §2.3.
    pub fn fresh() -> Self {
        Self(uuid::Uuid::now_v7().as_u128())
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn fresh_ids_are_unique() {
        let mut ids = HashSet::new();
        for _ in 0..100 {
            let id = RequestId::fresh();
            assert!(ids.insert(id), "duplicate RequestId {id:?}");
        }
        assert_eq!(ids.len(), 100);
    }
}
