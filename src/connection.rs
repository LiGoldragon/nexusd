//! `Connection` actor — per-client text shuttle.
//!
//! Reads nexus text from one client UDS to EOF, parses each
//! top-level expression into a [`signal::Request`] via
//! [`Parser`], opens a paired [`CriomeLink`] and forwards every
//! request, renders each reply back as text via [`Renderer`],
//! writes the accumulated text to the client, then stops self.
//!
//! M0 framing on the client side is one-shot: read until EOF,
//! process everything, write the rendered text, close. The
//! actor lifecycle therefore is single-message — `pre_start`
//! casts `Run`, `handle` does the shuttle, `myself.stop()`
//! ends the actor. M1+ streaming framing adds extra Message
//! variants without changing this skeleton.
//!
//! Parse errors before the first request abort the shuttle
//! before opening the criome link (no wasted connection); a
//! parse error mid-stream renders the error as a
//! `(Diagnostic …)` and closes — replies pair to requests by
//! position, so continuing past a parse error would lose sync.

use std::path::{Path, PathBuf};

use ractor::{Actor, ActorProcessingErr, ActorRef};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::criome_link::CriomeLink;
use crate::error::Result;
use crate::parser::Parser;
use crate::renderer::Renderer;

pub struct Connection;

pub struct State {
    client: UnixStream,
    criome_socket_path: PathBuf,
}

pub struct Arguments {
    pub client: UnixStream,
    pub criome_socket_path: PathBuf,
}

pub enum Message {
    /// One-shot tick that drives the entire shuttle. The actor
    /// stops itself after handling.
    Run,
}

impl State {
    async fn shuttle(&mut self) -> Result<()> {
        let mut text_input = String::new();
        self.client.read_to_string(&mut text_input).await?;

        let renderer = Self::process(&text_input, &self.criome_socket_path).await?;
        self.client.write_all(renderer.into_text().as_bytes()).await?;
        Ok(())
    }

    async fn process(text_input: &str, criome_socket_path: &Path) -> Result<Renderer> {
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

#[ractor::async_trait]
impl Actor for Connection {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        ractor::cast!(myself, Message::Run)?;
        Ok(State {
            client: arguments.client,
            criome_socket_path: arguments.criome_socket_path,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        _message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        if let Err(error) = state.shuttle().await {
            eprintln!("nexus-daemon: connection error: {error}");
        }
        myself.stop(None);
        Ok(())
    }
}
