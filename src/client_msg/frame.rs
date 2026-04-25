//! [`Frame`] — the wire envelope for any client ↔ nexusd.

use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};

use crate::client_msg::{Reply, Request};

/// Client-generated correlation ID.
///
/// Every client request carries one. nexusd never invents IDs
/// in this space; criome never sees them.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize,
    Clone, Copy, Debug, PartialEq, Eq, Hash,
)]
pub struct RequestId(pub u128);

impl RequestId {
    /// Generate a fresh client-side ID using UUID v7 (time-
    /// ordered, globally unique). Per
    /// `mentci-next/reports/071` §2.3.
    pub fn fresh() -> Self {
        Self(uuid::Uuid::now_v7().as_u128())
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub request_id: RequestId,
    pub body: Body,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Request(Request),
    Reply(Reply),
}

impl Frame {
    /// Encode to rkyv-archive bytes for socket write.
    ///
    /// rkyv 0.8 portable feature set per architecture.md §10
    /// guarantees bytes are deterministic across machines
    /// (little_endian + pointer_width_32 + unaligned).
    pub fn encode(&self) -> Vec<u8> {
        rkyv::to_bytes::<rkyv::rancor::Error>(self)
            .expect("rkyv serialisation does not fail for owned values")
            .to_vec()
    }

    /// Decode from rkyv-archive bytes off the socket. Validates
    /// the archive via `bytecheck` before deserialising.
    pub fn decode(bytes: &[u8]) -> Result<Self, FrameDecodeError> {
        rkyv::from_bytes::<Self, rkyv::rancor::Error>(bytes)
            .map_err(|_| FrameDecodeError::BadArchive)
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
    use crate::client_msg::{
        FallbackFormat, FallbackSpec, Reply, WirePath, WorkingStage,
    };
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

    #[test]
    fn frame_round_trip_send_with_fallback() {
        let original = Frame {
            request_id: RequestId(0x0123456789abcdef_fedcba9876543210),
            body: Body::Request(Request::Send {
                nexus_text: "(Assert (KindDecl :name \"KindDecl\"))".to_string(),
                fallback: Some(FallbackSpec {
                    path: WirePath(b"/tmp/example.nexus".to_vec()),
                    format: FallbackFormat::Nexus,
                }),
            }),
        };
        let bytes = original.encode();
        assert!(!bytes.is_empty());
        let decoded = Frame::decode(&bytes).expect("decode");
        assert_eq!(decoded, original);
    }

    #[test]
    fn frame_round_trip_unit_variants() {
        let cases = [
            Frame {
                request_id: RequestId::fresh(),
                body: Body::Request(Request::Heartbeat),
            },
            Frame {
                request_id: RequestId::fresh(),
                body: Body::Request(Request::Cancel),
            },
            Frame {
                request_id: RequestId::fresh(),
                body: Body::Reply(Reply::Ack),
            },
            Frame {
                request_id: RequestId::fresh(),
                body: Body::Reply(Reply::Cancelled),
            },
            Frame {
                request_id: RequestId::fresh(),
                body: Body::Reply(Reply::ResumeNotReady),
            },
        ];
        for f in cases {
            let bytes = f.encode();
            let decoded = Frame::decode(&bytes).expect("decode");
            assert_eq!(decoded, f);
        }
    }

    #[test]
    fn frame_round_trip_reply_working_stage() {
        for stage in [
            WorkingStage::Parsing,
            WorkingStage::AwaitingCriomed,
            WorkingStage::SerialisingReply,
        ] {
            let f = Frame {
                request_id: RequestId::fresh(),
                body: Body::Reply(Reply::Working { stage }),
            };
            let bytes = f.encode();
            let decoded = Frame::decode(&bytes).expect("decode");
            assert_eq!(decoded, f);
        }
    }

    #[test]
    fn decode_rejects_garbage() {
        let garbage = vec![0xff; 32];
        assert!(matches!(
            Frame::decode(&garbage),
            Err(FrameDecodeError::BadArchive)
        ));
    }

    #[test]
    fn wirepath_with_non_utf8_bytes_round_trips() {
        // POSIX paths can contain arbitrary bytes; the wire form
        // must not lose them.
        let f = Frame {
            request_id: RequestId::fresh(),
            body: Body::Reply(Reply::DoneWithFallback {
                reply_text: "ok".to_string(),
                fallback_path: WirePath(vec![b'/', 0xff, 0xfe, b'/', b'a']),
            }),
        };
        let bytes = f.encode();
        let decoded = Frame::decode(&bytes).expect("decode");
        assert_eq!(decoded, f);
    }
}
