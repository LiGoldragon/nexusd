//! `nexus-parse` — one-shot text-to-Frame parser.
//!
//! Read nexus text from stdin, emit length-prefixed
//! [`signal::Frame`]s to stdout — one frame per top-level
//! expression in the input. Useful in test pipelines, agent
//! harnesses, and debug scripts that want the parser without
//! running the daemon.

use std::io::{Read, Write};

use nexus::{Error, Parser, Result};
use signal::{Body, Frame};

fn main() -> Result<()> {
    let mut text = String::new();
    std::io::stdin().read_to_string(&mut text)?;

    let mut parser = Parser::new(&text);
    let mut stdout = std::io::stdout().lock();

    while let Some(request) = parser.next_request()? {
        let frame = Frame {
            principal_hint: None,
            auth_proof: None,
            body: Body::Request(request),
        };
        let bytes = frame.encode();
        let length = u32::try_from(bytes.len())
            .map_err(|_| Error::FrameTooLarge { length: bytes.len() })?;
        stdout.write_all(&length.to_be_bytes())?;
        stdout.write_all(&bytes)?;
    }

    Ok(())
}
