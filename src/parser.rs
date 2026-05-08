//! `Parser` — nexus text → [`signal::Request`].
//!
//! This is the current Criome-specific parser. It still accepts the
//! pre-renovation M0 surface while `nota-codec` / `nota-derive` are moved to
//! Nexus Tier 0. The Tier 0 target in `spec/grammar.md` is explicit
//! verb-record dispatch: `(Assert ...)`, `(Match ...)`, `(Subscribe ...)`,
//! and so on.
//!
//! Until the signal boundary is rebased onto the twelve-verb contract, this
//! parser remains a compatibility adapter: it parses old Assert forms and the
//! old pattern delimiter that still maps to `signal::Request::Query`. `Query`
//! is compatibility API naming, not a target Nexus verb; the Tier 0 target verb
//! is `Match`.
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
            Some(Token::LBracketPipe) => Err(Error::VerbNotInM0Scope { verb: "Atomic" }),
            Some(other) => Err(nota_codec::Error::UnexpectedToken {
                expected: "verb-opening sigil or delimiter — `(`, `(|`, `~`, `!`, `?`, `*`, or `[|`",
                got: other,
            }
            .into()),
        }
    }
}
