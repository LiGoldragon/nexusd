//! `Daemon` actor — root of the nexus supervision tree.
//!
//! Spawns the [`Listener`](crate::listener) at startup and
//! holds its `ActorRef` for graceful-shutdown propagation.

use std::path::PathBuf;

use kameo::actor::{Actor, ActorRef, Spawn, WeakActorRef};
use kameo::error::ActorStopReason;

use crate::listener;
use crate::{Error, Result};

pub struct Daemon {
    listener: ActorRef<listener::Listener>,
}

#[derive(Debug, Clone)]
pub struct Arguments {
    pub socket_path: PathBuf,
    pub criome_socket_path: PathBuf,
}

impl Daemon {
    pub async fn start(arguments: Arguments) -> Result<ActorRef<Self>> {
        let actor_reference = Self::spawn(arguments);
        actor_reference
            .wait_for_startup_with_result(|result| {
                result.map_err(|error| Error::ActorSpawn(format!("daemon startup: {error:?}")))
            })
            .await?;
        Ok(actor_reference)
    }
}

impl Actor for Daemon {
    type Args = Arguments;
    type Error = Error;

    async fn on_start(arguments: Self::Args, actor_reference: ActorRef<Self>) -> Result<Self> {
        let listener = listener::Listener::spawn_link(
            &actor_reference,
            listener::Arguments {
                socket_path: arguments.socket_path,
                criome_socket_path: arguments.criome_socket_path,
            },
        )
        .await;

        listener
            .wait_for_startup_with_result(|result| {
                result.map_err(|error| Error::ActorSpawn(format!("listener startup: {error:?}")))
            })
            .await?;

        Ok(Self { listener })
    }

    async fn on_stop(
        &mut self,
        _actor_reference: WeakActorRef<Self>,
        _reason: ActorStopReason,
    ) -> Result<()> {
        let _ = self.listener.stop_gracefully().await;
        self.listener.wait_for_shutdown().await;
        Ok(())
    }
}
