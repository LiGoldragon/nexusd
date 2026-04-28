//! `Daemon` actor — root of the nexus supervision tree.
//!
//! Spawns the [`Listener`](crate::listener) at startup and
//! holds its `ActorRef` for graceful-shutdown propagation.
//! The Daemon itself receives no user messages — it only
//! exists to own the supervision relationship and respond to
//! a `Stop` request from `main` (e.g., on SIGTERM). Bring it
//! up via `Actor::spawn(Some("daemon".into()), Daemon, args)`
//! at the binary entry point — see `bin/main.rs`.

use std::path::PathBuf;

use ractor::{Actor, ActorProcessingErr, ActorRef};

use crate::listener;

pub struct Daemon;

pub struct State {
    pub listener: ActorRef<listener::Message>,
}

pub struct Arguments {
    pub socket_path: PathBuf,
    pub criome_socket_path: PathBuf,
}

pub enum Message {}

#[ractor::async_trait]
impl Actor for Daemon {
    type Msg = Message;
    type State = State;
    type Arguments = Arguments;

    async fn pre_start(
        &self,
        myself: ActorRef<Self::Msg>,
        arguments: Arguments,
    ) -> std::result::Result<Self::State, ActorProcessingErr> {
        let (listener_ref, _) = Actor::spawn_linked(
            Some("listener".into()),
            listener::Listener,
            listener::Arguments {
                socket_path: arguments.socket_path,
                criome_socket_path: arguments.criome_socket_path,
            },
            myself.get_cell(),
        )
        .await?;

        Ok(State { listener: listener_ref })
    }

    async fn handle(
        &self,
        _myself: ActorRef<Self::Msg>,
        _message: Message,
        _state: &mut State,
    ) -> std::result::Result<(), ActorProcessingErr> {
        Ok(())
    }
}
