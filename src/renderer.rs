//! `Renderer` — [`signal::Reply`] → nexus text.
//!
//! Per-position dispatch on the reply variant; each reply
//! position produces one top-level text expression. Successive
//! replies are separated by `\n` so the client sees one
//! self-delimited expression per line.
//!
//! Per signal/ARCH the reply types are wire-only (no uniform
//! `NotaEncode` derive); rendering is application-specific to
//! the daemon. Data-record rendering (Node / Edge / Graph)
//! does go through the records' `NotaEncode` impl since those
//! types are part of the perfect-specificity schema and own
//! their own text form. The non-uniform variants —
//! `Diagnostic` and the `Handshake*` family — get
//! hand-rendered here.
//!
//! `Reply::HandshakeAccepted` / `HandshakeRejected` should
//! never reach the renderer in normal operation — the daemon
//! consumes them on the criome connection during its own
//! handshake. Encountering one in the user-visible reply
//! stream is a daemon protocol error.

use nota_codec::{Encoder, NotaEncode};
use signal::{Diagnostic, OutcomeMessage, Records, Reply};

use crate::error::{Error, Result};

pub struct Renderer {
    output: String,
}

impl Default for Renderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer {
    pub fn new() -> Self {
        Self { output: String::new() }
    }

    /// Append the rendered text of `reply` to the buffer.
    /// Inserts a `\n` separator before non-first replies.
    pub fn render_reply(&mut self, reply: &Reply) -> Result<()> {
        let mut encoder = Encoder::nexus();
        Self::render_into(reply, &mut encoder)?;
        if !self.output.is_empty() {
            self.output.push('\n');
        }
        self.output.push_str(&encoder.into_string());
        Ok(())
    }

    /// Consume the renderer and return the accumulated text.
    pub fn into_text(self) -> String {
        self.output
    }

    fn render_into(reply: &Reply, encoder: &mut Encoder) -> Result<()> {
        match reply {
            Reply::Outcome(outcome) => Self::render_outcome(outcome, encoder),
            Reply::Outcomes(outcomes) => {
                encoder.start_seq()?;
                for outcome in outcomes {
                    Self::render_outcome(outcome, encoder)?;
                }
                encoder.end_seq()?;
                Ok(())
            }
            Reply::Records(records) => Self::render_records(records, encoder),
            Reply::HandshakeAccepted(_) => {
                Err(Error::HandshakePostReplyShape { got: "HandshakeAccepted" })
            }
            Reply::HandshakeRejected(_) => {
                Err(Error::HandshakePostReplyShape { got: "HandshakeRejected" })
            }
        }
    }

    fn render_outcome(outcome: &OutcomeMessage, encoder: &mut Encoder) -> Result<()> {
        match outcome {
            OutcomeMessage::Ok(ok) => Ok(ok.encode(encoder)?),
            OutcomeMessage::Diagnostic(diagnostic) => {
                Self::render_diagnostic(diagnostic, encoder)
            }
        }
    }

    /// `(Diagnostic <Level> <code> <message>)`. The full
    /// Diagnostic shape (primary_site / context / suggestions /
    /// durable_record) is omitted in M0; richer rendering lands
    /// when those fields actually carry information.
    fn render_diagnostic(diagnostic: &Diagnostic, encoder: &mut Encoder) -> Result<()> {
        encoder.start_record("Diagnostic")?;
        diagnostic.level.encode(encoder)?;
        encoder.write_string(&diagnostic.code)?;
        encoder.write_string(&diagnostic.message)?;
        encoder.end_record()?;
        Ok(())
    }

    fn render_records(records: &Records, encoder: &mut Encoder) -> Result<()> {
        match records {
            Records::Node(items) => Self::render_record_seq(items, encoder),
            Records::Edge(items) => Self::render_record_seq(items, encoder),
            Records::Graph(items) => Self::render_record_seq(items, encoder),
        }
    }

    fn render_record_seq<T: NotaEncode>(items: &[T], encoder: &mut Encoder) -> Result<()> {
        encoder.start_seq()?;
        for item in items {
            item.encode(encoder)?;
        }
        encoder.end_seq()?;
        Ok(())
    }

    /// Render a daemon-side error (parser failure, internal
    /// error) directly as a `(Diagnostic …)` text reply. Used
    /// when the parser rejects user text before the request
    /// can reach criome.
    pub fn render_local_error(&mut self, error: &Error) -> Result<()> {
        let mut encoder = Encoder::nexus();
        encoder.start_record("Diagnostic")?;
        encoder.write_pascal_identifier("Error")?;
        encoder.write_string(Self::local_error_code(error))?;
        encoder.write_string(&error.to_string())?;
        encoder.end_record()?;
        if !self.output.is_empty() {
            self.output.push('\n');
        }
        self.output.push_str(&encoder.into_string());
        Ok(())
    }

    fn local_error_code(error: &Error) -> &'static str {
        match error {
            Error::Codec(_) => "E0001",
            Error::VerbNotInM0Scope { .. } => "E0099",
            Error::Io(_) => "E0010",
            Error::Frame(_) => "E0011",
            Error::FrameTooLarge { .. } => "E0012",
            Error::HandshakeRejected { .. } => "E0020",
            Error::HandshakePostReplyShape { .. } => "E0021",
            Error::ActorCall(_) => "E0030",
            Error::ActorSpawn(_) => "E0031",
        }
    }
}
