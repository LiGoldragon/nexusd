//! `Listener` actor — UDS accept loop.
//!
//! Binds the user-facing nexus socket and accepts client
//! connections, spawning one [`Connection`](crate::connection)
//! actor per accept. Holds the criome-side socket path so it
//! can pass it into each spawned Connection — each connection
//! opens its own paired [`CriomeLink`](crate::criome_link::CriomeLink)
//! lazily on first request.

use std::path::PathBuf;

use kameo::actor::{Actor, ActorRef, Spawn};
use kameo::message::{Context, Message};
use tokio::net::UnixListener;

use crate::connection;
use crate::{Error, Result};

pub struct Listener {
    listener: UnixListener,
    criome_socket_path: PathBuf,
    accepted_connections: u64,
}

#[derive(Debug, Clone)]
pub struct Arguments {
    pub socket_path: PathBuf,
    pub criome_socket_path: PathBuf,
}

pub struct AcceptConnection;

impl Listener {
    async fn accept_connection(&mut self, listener_ref: &ActorRef<Self>) -> Result<()> {
        let (client, _) = self.listener.accept().await?;
        let connection = connection::Connection::spawn_link(
            listener_ref,
            connection::Arguments {
                client,
                criome_socket_path: self.criome_socket_path.clone(),
            },
        )
        .await;
        connection
            .wait_for_startup_with_result(|result| {
                result.map_err(|error| Error::ActorSpawn(format!("connection startup: {error:?}")))
            })
            .await?;
        self.accepted_connections = self.accepted_connections.saturating_add(1);
        Ok(())
    }
}

impl Actor for Listener {
    type Args = Arguments;
    type Error = Error;

    async fn on_start(arguments: Self::Args, actor_reference: ActorRef<Self>) -> Result<Self> {
        let _ = std::fs::remove_file(&arguments.socket_path);
        let listener = UnixListener::bind(&arguments.socket_path)?;
        actor_reference
            .tell(AcceptConnection)
            .await
            .map_err(|error| Error::ActorCall(error.to_string()))?;
        Ok(Self {
            listener,
            criome_socket_path: arguments.criome_socket_path,
            accepted_connections: 0,
        })
    }
}

impl Message<AcceptConnection> for Listener {
    type Reply = ();

    async fn handle(
        &mut self,
        _message: AcceptConnection,
        context: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        match self.accept_connection(context.actor_ref()).await {
            Ok(()) => {
                if let Err(error) = context.actor_ref().tell(AcceptConnection).await {
                    eprintln!("nexus-daemon: listener could not re-arm accept loop: {error}");
                    context.stop();
                }
            }
            Err(error) => {
                eprintln!("nexus-daemon: listener accept failed: {error}");
                context.stop();
            }
        }
    }
}
