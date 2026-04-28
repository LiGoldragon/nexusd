//! `Listener` actor — UDS accept loop.
//!
//! Binds the user-facing nexus socket and accepts client
//! connections, spawning one [`Connection`](crate::connection)
//! actor per accept. Holds the criome-side socket path so it
//! can pass it into each spawned Connection — each connection
//! opens its own paired [`CriomeLink`](crate::criome_link::CriomeLink)
//! lazily on first request.
//!
//! The accept loop is modeled as a self-cast `Accept` message
//! — each tick accepts one connection, spawns the child, and
//! re-arms. Connection panics are logged and the listener
//! moves on.

use std::path::PathBuf;

use ractor::{Actor, ActorProcessingErr, ActorRef, SupervisionEvent};
use tokio::net::UnixListener;

use crate::connection;

pub struct Listener;

pub struct State {
    listener: UnixListener,
    criome_socket_path: PathBuf,
}

pub struct Arguments {
    pub socket_path: PathBuf,
    pub criome_socket_path: PathBuf,
}

pub enum Message {
    /// Self-cast tick that accepts one connection per
    /// invocation and re-arms.
    Accept,
}

#[ractor::async_trait]
impl Actor for Listener {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        let _ = std::fs::remove_file(&arguments.socket_path);
        let listener = UnixListener::bind(&arguments.socket_path)?;
        ractor::cast!(myself, Message::Accept)?;
        Ok(State {
            listener,
            criome_socket_path: arguments.criome_socket_path,
        })
    }

    async fn handle(
        &self,
        myself: ActorRef<Self::Msg>,
        message: Message,
        state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        match message {
            Message::Accept => {
                let (client, _) = state.listener.accept().await?;
                let arguments = connection::Arguments {
                    client,
                    criome_socket_path: state.criome_socket_path.clone(),
                };
                Actor::spawn_linked(None, connection::Connection, arguments, myself.get_cell())
                    .await?;
                ractor::cast!(myself, Message::Accept)?;
            }
        }
        Ok(())
    }

    async fn handle_supervisor_evt(
        &self,
        _myself: ActorRef<Self::Msg>,
        event: SupervisionEvent,
        _state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        if let SupervisionEvent::ActorFailed(actor, reason) = event {
            eprintln!("nexus-daemon: connection {actor:?} failed: {reason}");
        }
        Ok(())
    }
}
