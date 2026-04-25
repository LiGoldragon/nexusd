//! [`FallbackSpec`] — where nexusd writes a reply if the client
//! socket disappears before the reply is ready.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FallbackSpec {
    /// Filesystem path nexusd will write to. Atomic rename
    /// (write-temp + rename) so a polling client never observes a
    /// half-written file.
    pub path: PathBuf,

    /// What format to serialise the reply as.
    pub format: FallbackFormat,
}

#[derive(Debug, Clone, Copy)]
pub enum FallbackFormat {
    /// Human-readable nexus text. Same content as `CliReply::Done {
    /// reply_text }`.
    Nexus,

    /// rkyv-archived `CliReply` so a resuming client can decode it
    /// the same way it would a live wire reply.
    Rkyv,
}
