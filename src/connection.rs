//! `Connection` — a per-client text-shuttle session.
//!
//! Reads nexus text from the client socket, parses each
//! top-level expression into a [`signal::Request`], forwards
//! every request through a paired [`CriomeLink`], renders each
//! reply back as text, and writes the accumulated text to the
//! client.
//!
//! M0 framing on the client side is one-shot: the daemon reads
//! until EOF, processes everything, writes the rendered text,
//! and closes. M1+ may add streaming framing.
//!
//! Parse errors before the first request abort the shuttle
//! before opening the criome link (no wasted connection); a
//! parse error mid-stream renders the error as a
//! `(Diagnostic …)` and closes — replies pair to requests by
//! position, so continuing past a parse error would lose sync.

use std::path::PathBuf;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::criome_link::CriomeLink;
use crate::error::Result;
use crate::parser::Parser;
use crate::renderer::Renderer;

pub struct Connection {
    client: UnixStream,
    criome_socket_path: PathBuf,
}

impl Connection {
    pub fn new(client: UnixStream, criome_socket_path: PathBuf) -> Self {
        Self { client, criome_socket_path }
    }

    pub async fn shuttle(mut self) -> Result<()> {
        let mut text_input = String::new();
        self.client.read_to_string(&mut text_input).await?;

        let renderer = Self::process(&text_input, &self.criome_socket_path).await?;
        self.client.write_all(renderer.into_text().as_bytes()).await?;
        Ok(())
    }

    async fn process(text_input: &str, criome_socket_path: &PathBuf) -> Result<Renderer> {
        let mut renderer = Renderer::new();
        let mut parser = Parser::new(text_input);

        let first = match parser.next_request() {
            Ok(Some(request)) => request,
            Ok(None) => return Ok(renderer),
            Err(parse_error) => {
                renderer.render_local_error(&parse_error)?;
                return Ok(renderer);
            }
        };

        let mut criome = CriomeLink::open(criome_socket_path).await?;
        renderer.render_reply(&criome.send(first).await?)?;

        loop {
            match parser.next_request() {
                Ok(None) => return Ok(renderer),
                Ok(Some(request)) => {
                    renderer.render_reply(&criome.send(request).await?)?;
                }
                Err(parse_error) => {
                    renderer.render_local_error(&parse_error)?;
                    return Ok(renderer);
                }
            }
        }
    }
}
