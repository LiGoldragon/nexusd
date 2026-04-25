//! [`CliReply`] — what nexusd sends back to nexus-cli.

use std::path::PathBuf;

use crate::cli_msg::CliRequestId;

#[derive(Debug)]
pub enum CliReply {
    /// `Send` received and accepted. Subsequent state is
    /// reportable via `Working` / `Done`.
    Ack,

    /// Probe response: still working. `stage` lets the client
    /// distinguish parse-stage from criomed-pending from
    /// reply-serialisation.
    Working { stage: WorkingStage },

    /// The request completed.
    Done { reply_text: String },

    /// The request completed AND nexusd also wrote the reply to
    /// the fallback path the client supplied. Useful when the
    /// requester's socket has been flaky and the client wants
    /// belt-and-braces delivery.
    DoneWithFallback {
        reply_text: String,
        fallback_path: PathBuf,
    },

    /// Failed before reaching criomed (parse error, schema
    /// rejection at the syntactic layer, transport failure to
    /// criomed). For criomed-validated rejections, the reply
    /// text carries a structured Diagnostic in the criome-msg
    /// layer; this variant is for nexusd-internal failures.
    Failed { error: String },

    /// Cancellation acknowledged.
    Cancelled,

    /// Resume succeeded; here is the reply from your earlier
    /// request, retrieved from the fallback path. nexusd has
    /// deleted the file.
    ResumedReply {
        original_request_id: CliRequestId,
        reply_text: String,
    },

    /// Resume failed — the fallback path was empty or did not
    /// exist. The original request may still be in flight or
    /// may have been processed before fallback was written.
    ResumeNotReady,

    /// Reply was computed but the fallback write failed (disk
    /// full, permission denied, etc.). The reply text is
    /// still here — the client can choose to act on it without
    /// the fallback artifact.
    FailedFallback {
        reply_text: String,
        fallback_error: String,
    },
}

#[derive(Debug)]
pub enum WorkingStage {
    /// Still parsing the nexus text.
    Parsing,

    /// Waiting on criomed (criome-msg request sent, reply not yet
    /// in).
    AwaitingCriomed,

    /// Got criomed's reply; serialising back to nexus text.
    SerialisingReply,
}
