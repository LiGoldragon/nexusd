//! `nexus-render` — one-shot Frame-to-text renderer.
//!
//! Read length-prefixed [`signal::Frame`]s from stdin (one or
//! more), render each via [`nexus::Renderer`], emit the
//! accumulated text to stdout. Multiple replies are
//! newline-separated per Renderer's discipline.

use std::io::{Read, Write};

use nexus::{Error, Renderer, Result};
use signal::{Body, Frame};

fn main() -> Result<()> {
    let mut stdin = std::io::stdin().lock();
    let mut renderer = Renderer::new();

    loop {
        let mut length_bytes = [0u8; 4];
        match stdin.read_exact(&mut length_bytes) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => return Err(Error::Io(error)),
        }
        let length = u32::from_be_bytes(length_bytes) as usize;
        let mut frame_bytes = vec![0u8; length];
        stdin.read_exact(&mut frame_bytes)?;

        let frame = Frame::decode(&frame_bytes)?;
        match frame.body {
            Body::Reply(reply) => renderer.render_reply(&reply)?,
            Body::Request(_) => {
                return Err(Error::HandshakePostReplyShape {
                    got: "Body::Request",
                });
            }
        }
    }

    let mut stdout = std::io::stdout().lock();
    write!(stdout, "{}", renderer.into_text())?;
    Ok(())
}
