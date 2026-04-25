//! [`CliRequest`] — what nexus-cli sends to nexusd.

use crate::cli_msg::FallbackSpec;

#[derive(Debug)]
pub enum CliRequest {
    /// Submit a nexus message. nexusd parses it, builds a
    /// criome-msg envelope, forwards to criomed, awaits reply,
    /// returns it as a [`super::CliReply::Done`].
    Send {
        /// Raw nexus text. nexusd is responsible for parsing.
        nexus_text: String,

        /// Optional path for nexusd to write the reply to if the
        /// requester's socket disappears before the reply is
        /// ready. A later `Resume` can pick the reply up from
        /// this path.
        fallback: Option<FallbackSpec>,
    },

    /// "Still waiting, is everything ok?" — sent periodically by
    /// nexus-cli while a `Send` is outstanding. nexusd replies
    /// with [`super::CliReply::Working`] or
    /// [`super::CliReply::Done`] depending on state.
    ///
    /// Carries no extra payload; the `cli_request_id` in the
    /// enclosing [`super::CliFrame`] is the correlation.
    Heartbeat,

    /// Cancel an in-flight request. nexusd may forward a cancel
    /// to criomed if the criome-msg layer supports it; otherwise
    /// the reply (if any) is discarded.
    Cancel,

    /// Resume an earlier request whose reply was written to a
    /// fallback path because the original requester's socket
    /// dropped. nexusd reads the path, returns its contents as
    /// [`super::CliReply::ResumedReply`], deletes the file.
    ///
    /// The `cli_request_id` carried on the enclosing frame is the
    /// id of *this* resume request, not the original. The
    /// original ID is in the resume payload.
    Resume {
        original_request_id: super::CliRequestId,
        fallback: FallbackSpec,
    },
}
