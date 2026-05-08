//! `CriomeLink` — a post-handshake signal connection to criome.
//!
//! Encapsulates the wire framing (4-byte big-endian length
//! prefix + N rkyv bytes per [`Frame`], per signal/ARCH §"Wire
//! format") and the invariant that the handshake has already
//! succeeded. Once `CriomeLink::open` returns `Ok`, the link
//! is ready for verb requests; `send` writes one request frame
//! and reads one reply frame, paired by FIFO position per
//! signal/ARCH §"Reply protocol".

use std::path::Path;

use signal::{Body, Frame, HandshakeRequest, Reply, Request, SIGNAL_PROTOCOL_VERSION};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::error::{Error, Result};

const CLIENT_NAME: &str = "nexus";

pub struct CriomeLink {
    stream: UnixStream,
}

impl CriomeLink {
    /// Open a connection to criome at `socket_path` and complete
    /// the protocol handshake. Returns the post-handshake link
    /// ready for verb requests.
    pub async fn open(socket_path: &Path) -> Result<Self> {
        let stream = UnixStream::connect(socket_path).await?;
        let mut link = Self { stream };
        link.handshake().await?;
        Ok(link)
    }

    /// Send one request and read its paired reply. Replies pair
    /// by position (FIFO) — the next reply on the link
    /// corresponds to the request just sent.
    pub async fn send(&mut self, request: Request) -> Result<Reply> {
        let request_frame = Frame {
            principal_hint: None,
            auth_proof: None,
            body: Body::Request(request),
        };
        self.write_frame(&request_frame).await?;
        let reply_frame = self.read_frame().await?;
        match reply_frame.body {
            Body::Reply(reply) => Ok(reply),
            Body::Request(_) => Err(Error::HandshakePostReplyShape {
                got: "Body::Request",
            }),
        }
    }

    async fn handshake(&mut self) -> Result<()> {
        let request_frame = Frame {
            principal_hint: None,
            auth_proof: None,
            body: Body::Request(Request::Handshake(HandshakeRequest {
                client_version: SIGNAL_PROTOCOL_VERSION,
                client_name: CLIENT_NAME.to_string(),
            })),
        };
        self.write_frame(&request_frame).await?;
        let reply_frame = self.read_frame().await?;
        match reply_frame.body {
            Body::Reply(Reply::HandshakeAccepted(_)) => Ok(()),
            Body::Reply(Reply::HandshakeRejected(reason)) => {
                Err(Error::HandshakeRejected { reason })
            }
            Body::Reply(Reply::Outcome(_)) => Err(Error::HandshakePostReplyShape {
                got: "Reply::Outcome",
            }),
            Body::Reply(Reply::Outcomes(_)) => Err(Error::HandshakePostReplyShape {
                got: "Reply::Outcomes",
            }),
            Body::Reply(Reply::Records(_)) => Err(Error::HandshakePostReplyShape {
                got: "Reply::Records",
            }),
            Body::Request(_) => Err(Error::HandshakePostReplyShape {
                got: "Body::Request",
            }),
        }
    }

    async fn write_frame(&mut self, frame: &Frame) -> Result<()> {
        let bytes = frame.encode();
        let length = u32::try_from(bytes.len()).map_err(|_| Error::FrameTooLarge {
            length: bytes.len(),
        })?;
        self.stream.write_all(&length.to_be_bytes()).await?;
        self.stream.write_all(&bytes).await?;
        Ok(())
    }

    async fn read_frame(&mut self) -> Result<Frame> {
        let mut length_bytes = [0u8; 4];
        self.stream.read_exact(&mut length_bytes).await?;
        let length = u32::from_be_bytes(length_bytes) as usize;
        let mut bytes = vec![0u8; length];
        self.stream.read_exact(&mut bytes).await?;
        Ok(Frame::decode(&bytes)?)
    }
}
