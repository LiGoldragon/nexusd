//! `Parser` — nexus text → [`signal::Request`].
//!
//! This is the current Criome-specific parser. It still accepts only the
//! pre-renovation Assert surface while `signal` is moved to the twelve-verb
//! contract. The Tier 0 target in `spec/grammar.md` is explicit verb-record
//! dispatch: `(Assert ...)`, `(Match ...)`, `(Subscribe ...)`, and so on.
//!
//! Until the signal boundary is rebased onto the twelve-verb contract, this
//! parser remains a compatibility adapter. Retired Nexus sigils and piped
//! delimiters are not tokenized by nota-codec anymore, so this parser does not
//! preserve the old `(| ... |)` query form.
//!
//! `Request::Handshake` does not appear in user-facing text —
//! the daemon performs the handshake with criome internally
//! when opening each per-connection signal session.

use nota_codec::{Decoder, NotaDecode, Token};
use signal::{AssertOperation, Request};

use crate::error::Result;

pub struct Parser<'input> {
    decoder: Decoder<'input>,
}

impl<'input> Parser<'input> {
    /// Open a parser over a slice of nexus text.
    pub fn new(input: &'input str) -> Self {
        Self {
            decoder: Decoder::new(input),
        }
    }

    /// Read the next top-level request, or `None` at end of
    /// input. Errors leave the decoder mid-stream — the caller
    /// stops on the first error since reply positions would
    /// otherwise lose sync with the surviving requests.
    pub fn next_request(&mut self) -> Result<Option<Request>> {
        match self.decoder.peek_token()? {
            None => Ok(None),
            Some(Token::LParen) => {
                let operation = AssertOperation::decode(&mut self.decoder)?;
                Ok(Some(Request::Assert(operation)))
            }
            Some(other) => Err(nota_codec::Error::UnexpectedToken {
                expected: "`(` opening the current compatibility Assert record",
                got: other,
            }
            .into()),
        }
    }
}
