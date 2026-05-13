//! `Connection` actor — per-client text shuttle.
//!
//! Reads nexus text from one client UDS to EOF, parses each
//! top-level expression into a [`signal::Request`] via
//! [`Parser`], opens a paired [`CriomeLink`] and forwards every
//! request, renders each reply back as text via [`Renderer`],
//! writes the accumulated text to the client, then stops self.

use std::path::{Path, PathBuf};

use kameo::actor::{Actor, ActorRef};
use kameo::message::{Context, Message};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;

use crate::Error;
use crate::criome_link::CriomeLink;
use crate::error::Result;
use crate::parser::Parser;
use crate::renderer::Renderer;

pub struct Connection {
    client: UnixStream,
    criome_socket_path: PathBuf,
    handled_requests: u64,
}

pub struct Arguments {
    pub client: UnixStream,
    pub criome_socket_path: PathBuf,
}

pub struct RunConnection;

impl Connection {
    async fn shuttle(&mut self) -> Result<()> {
        let mut text_input = String::new();
        self.client.read_to_string(&mut text_input).await?;

        let (renderer, handled_requests) =
            Self::process(&text_input, &self.criome_socket_path).await?;
        self.handled_requests = self.handled_requests.saturating_add(handled_requests);
        self.client
            .write_all(renderer.into_text().as_bytes())
            .await?;
        Ok(())
    }

    async fn process(text_input: &str, criome_socket_path: &Path) -> Result<(Renderer, u64)> {
        let mut renderer = Renderer::new();
        let mut parser = Parser::new(text_input);
        let mut handled_requests = 0_u64;

        let first = match parser.next_request() {
            Ok(Some(request)) => request,
            Ok(None) => return Ok((renderer, handled_requests)),
            Err(parse_error) => {
                renderer.render_local_error(&parse_error)?;
                return Ok((renderer, handled_requests));
            }
        };

        let mut criome = CriomeLink::open(criome_socket_path).await?;
        renderer.render_reply(&criome.send(first).await?)?;
        handled_requests = handled_requests.saturating_add(1);

        loop {
            match parser.next_request() {
                Ok(None) => return Ok((renderer, handled_requests)),
                Ok(Some(request)) => {
                    renderer.render_reply(&criome.send(request).await?)?;
                    handled_requests = handled_requests.saturating_add(1);
                }
                Err(parse_error) => {
                    renderer.render_local_error(&parse_error)?;
                    return Ok((renderer, handled_requests));
                }
            }
        }
    }
}

impl Actor for Connection {
    type Args = Arguments;
    type Error = Error;

    async fn on_start(arguments: Self::Args, actor_reference: ActorRef<Self>) -> Result<Self> {
        actor_reference
            .tell(RunConnection)
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        Ok(Self {
            client: arguments.client,
            criome_socket_path: arguments.criome_socket_path,
            handled_requests: 0,
        })
    }
}

impl Message<RunConnection> for Connection {
    type Reply = ();

    async fn handle(
        &mut self,
        _message: RunConnection,
        context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        if let Err(error) = self.shuttle().await {
            eprintln!("nexus-daemon: connection error: {error}");
        }
        let _ = self.client.shutdown().await;
        context.stop();
    }
}
