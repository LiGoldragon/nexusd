//! `Parser` — nexus text → [`signal::Request`].
//!
//! Top-level dispatch over the verb-opening sigils and
//! delimiters defined in
//! [nexus/spec/grammar.md](../spec/grammar.md). The verb is
//! determined by the first token of each top-level expression:
//!
//! | First token | Verb | Wire path |
//! |---|---|---|
//! | `(` | Assert | `AssertOperation::decode` |
//! | `(\|` | Query | `QueryOperation::decode` |
//! | `~` | Mutate / Mutate-with-pattern | M1+ |
//! | `!` | Retract / Retract-matching | M1+ |
//! | `?` | Validate | M1+ |
//! | `*` | Subscribe | M2+ |
//! | `[\|` | AtomicBatch | M1+ |
//!
//! M0 parses Assert and Query directly; the other verbs return
//! [`Error::VerbNotInM0Scope`]. The renderer turns that error
//! into a `(Diagnostic …)` text reply for the user.
//!
//! `Request::Handshake` does not appear in user-facing text —
//! the daemon performs the handshake with criome internally
//! when opening each per-connection signal session.

use nota_codec::{Decoder, NotaDecode, Token};
use signal::{AssertOperation, QueryOperation, Request};

use crate::error::{Error, Result};

pub struct Parser<'input> {
    decoder: Decoder<'input>,
}

impl<'input> Parser<'input> {
    /// Open a parser over a slice of nexus text.
    pub fn new(input: &'input str) -> Self {
        Self { decoder: Decoder::nexus(input) }
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
            Some(Token::LParenPipe) => {
                let operation = QueryOperation::decode(&mut self.decoder)?;
                Ok(Some(Request::Query(operation)))
            }
            Some(Token::Tilde) => Err(Error::VerbNotInM0Scope { verb: "Mutate" }),
            Some(Token::Bang) => Err(Error::VerbNotInM0Scope { verb: "Retract" }),
            Some(Token::Question) => Err(Error::VerbNotInM0Scope { verb: "Validate" }),
            Some(Token::Star) => Err(Error::VerbNotInM0Scope { verb: "Subscribe" }),
            Some(Token::LBracketPipe) => Err(Error::VerbNotInM0Scope { verb: "AtomicBatch" }),
            Some(other) => Err(nota_codec::Error::UnexpectedToken {
                expected: "verb-opening sigil or delimiter — `(`, `(|`, `~`, `!`, `?`, `*`, or `[|`",
                got: other,
            }
            .into()),
        }
    }
}
