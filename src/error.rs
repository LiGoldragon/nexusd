//! Daemon-specific errors. Most parse-side errors used to live
//! here for the hand-written QueryParser; that parser was
//! deleted when [`nota_codec`]'s `NexusPattern` derive landed.
//! The remaining variants cover daemon-process errors (i/o,
//! handshake-rejection, criome-side dispatch failures); the
//! codec carries its own typed `nota_codec::Error` for
//! parse/encode failures.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),

    #[error("nota-codec: {0}")]
    Codec(#[from] nota_codec::Error),

    #[error("frame decode: {0}")]
    Frame(#[from] signal::FrameDecodeError),
}

pub type Result<T> = std::result::Result<T, Error>;
