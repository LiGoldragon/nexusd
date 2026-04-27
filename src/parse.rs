//! Query parser — `(| Kind ... |)` text → typed `signal::QueryOp`.
//!
//! The pattern container `(| ... |)` is parsed here in the daemon
//! rather than through serde's enum-by-name dispatch. Each field
//! position is one of three forms:
//!
//! - `_`            → `PatternField::Wildcard`
//! - `@<name>`      → `PatternField::Bind` (name MUST equal the
//!                    schema field name at this position; the
//!                    parser validates this — see
//!                    [nexus/spec/grammar.md §Binds][grammar])
//! - `<literal>`    → `PatternField::Match(value)` where value is
//!                    parsed as the field's typed Rust value
//!
//! Per the [perfect-specificity invariant][invariant-d], every
//! kind has a hand-written arm here today; rsc will generate
//! these arms from `KindDecl` records once it lands.
//!
//! [grammar]: https://github.com/LiGoldragon/nexus/blob/main/spec/grammar.md
//! [invariant-d]: https://github.com/LiGoldragon/criome/blob/main/ARCHITECTURE.md#invariant-d

use nota_serde_core::{is_lowercase_identifier, is_pascal_case, Lexer, Token};
use signal::{
    EdgeQuery, GraphQuery, KindDeclQuery, NodeQuery, PatternField, QueryOp, RelationKind, Slot,
};

use crate::error::{Error, Result};

/// Parses nexus-text query expressions into typed `QueryOp` values.
///
/// Owns a [`Lexer`] over a borrowed input string. Two consumption
/// patterns:
///
/// - **One-shot**: `QueryParser::new(input).into_query()` parses a
///   single query and rejects any trailing content.
/// - **Streaming**: hold the parser and call `parse` repeatedly to
///   pull queries off a multi-expression input. Each call advances
///   the underlying lexer.
pub struct QueryParser<'input> {
    lexer: Lexer<'input>,
}

impl<'input> QueryParser<'input> {
    /// Construct a parser over the given nexus-text input.
    pub fn new(input: &'input str) -> Self {
        Self { lexer: Lexer::nexus(input) }
    }

    /// Consume the parser and return the single query the input
    /// contains. Trailing content (anything after the closing
    /// `|)`) is an error.
    pub fn into_query(mut self) -> Result<QueryOp> {
        let query = self.parse()?;
        if let Some(trailing) = self.next_token()? {
            return Err(Error::TrailingTokens {
                got: format!("{trailing:?}"),
            });
        }
        Ok(query)
    }

    /// Parse the next `(| Kind ... |)` query in the lexer's
    /// stream. Leaves the lexer positioned just after the
    /// closing `|)`, ready for the next expression.
    pub fn parse(&mut self) -> Result<QueryOp> {
        self.expect_token(&Token::LParenPipe)?;
        let kind_name = self.expect_pascal_identifier()?;

        let query = match kind_name.as_str() {
            "Node" => QueryOp::Node(NodeQuery {
                name: self.parse_pattern_field_string("name")?,
            }),
            "Edge" => QueryOp::Edge(EdgeQuery {
                from: self.parse_pattern_field_slot("from")?,
                to: self.parse_pattern_field_slot("to")?,
                kind: self.parse_pattern_field_relation_kind("kind")?,
            }),
            "Graph" => QueryOp::Graph(GraphQuery {
                title: self.parse_pattern_field_string("title")?,
            }),
            "KindDecl" => QueryOp::KindDecl(KindDeclQuery {
                name: self.parse_pattern_field_string("name")?,
            }),
            other => {
                return Err(Error::UnknownQueryKind {
                    kind_name: other.to_string(),
                });
            }
        };

        self.expect_token(&Token::RParenPipe)?;
        Ok(query)
    }

    /// Parse a `PatternField<String>` at the current position.
    /// `expected_field_name` is the schema field name; bind names
    /// that don't match are rejected. The `Match` variant accepts
    /// both bare-identifier strings (`Alice`) and quoted strings
    /// (`"Alice"`).
    fn parse_pattern_field_string(
        &mut self,
        expected_field_name: &str,
    ) -> Result<PatternField<String>> {
        match self.next_token()? {
            Some(Token::Ident(text)) if text == "_" => Ok(PatternField::Wildcard),
            Some(Token::At) => {
                let bind_name = self.expect_lowercase_identifier()?;
                check_bind_name(&bind_name, expected_field_name)?;
                Ok(PatternField::Bind)
            }
            Some(Token::Ident(text)) => Ok(PatternField::Match(text)),
            Some(Token::Str(text)) => Ok(PatternField::Match(text)),
            other => Err(Error::ExpectedPatternField {
                got: format!("{other:?}"),
            }),
        }
    }

    /// Parse a `PatternField<Slot>` at the current position. Match
    /// values are bare integers (per the `#[serde(transparent)]`
    /// rule on `Slot` — see nota README §Newtype structs).
    fn parse_pattern_field_slot(
        &mut self,
        expected_field_name: &str,
    ) -> Result<PatternField<Slot>> {
        match self.next_token()? {
            Some(Token::Ident(text)) if text == "_" => Ok(PatternField::Wildcard),
            Some(Token::At) => {
                let bind_name = self.expect_lowercase_identifier()?;
                check_bind_name(&bind_name, expected_field_name)?;
                Ok(PatternField::Bind)
            }
            Some(Token::Int(value)) if value >= 0 => {
                Ok(PatternField::Match(Slot(value as u64)))
            }
            Some(Token::UInt(value)) => Ok(PatternField::Match(Slot(value as u64))),
            other => Err(Error::ExpectedPatternField {
                got: format!("{other:?}"),
            }),
        }
    }

    /// Parse a `PatternField<RelationKind>` at the current
    /// position. Match values are bare PascalCase variant names
    /// (`DependsOn`, `Flow`, etc).
    fn parse_pattern_field_relation_kind(
        &mut self,
        expected_field_name: &str,
    ) -> Result<PatternField<RelationKind>> {
        match self.next_token()? {
            Some(Token::Ident(text)) if text == "_" => Ok(PatternField::Wildcard),
            Some(Token::At) => {
                let bind_name = self.expect_lowercase_identifier()?;
                check_bind_name(&bind_name, expected_field_name)?;
                Ok(PatternField::Bind)
            }
            Some(Token::Ident(variant_name)) => {
                let variant = RelationKind::from_variant_name(&variant_name).ok_or_else(|| {
                    Error::UnknownRelationKindVariant {
                        expected_variants: RelationKind::ALL
                            .iter()
                            .map(|kind| kind.variant_name())
                            .collect::<Vec<_>>()
                            .join("|"),
                        got: variant_name.clone(),
                    }
                })?;
                Ok(PatternField::Match(variant))
            }
            other => Err(Error::ExpectedPatternField {
                got: format!("{other:?}"),
            }),
        }
    }

    fn next_token(&mut self) -> Result<Option<Token>> {
        self.lexer
            .next_token()
            .map_err(|err| Error::Lex(err.to_string()))
    }

    fn expect_token(&mut self, expected: &Token) -> Result<()> {
        match self.next_token()? {
            Some(token) if &token == expected => Ok(()),
            Some(other) => Err(Error::UnexpectedToken {
                expected: format!("{expected:?}"),
                got: format!("{other:?}"),
            }),
            None => Err(Error::UnexpectedToken {
                expected: format!("{expected:?}"),
                got: "end of input".to_string(),
            }),
        }
    }

    fn expect_pascal_identifier(&mut self) -> Result<String> {
        match self.next_token()? {
            Some(Token::Ident(text)) => {
                if !is_pascal_case(&text) {
                    return Err(Error::ExpectedPascalIdentifier {
                        got: format!("Ident({text:?})"),
                    });
                }
                Ok(text)
            }
            other => Err(Error::ExpectedPascalIdentifier {
                got: format!("{other:?}"),
            }),
        }
    }

    fn expect_lowercase_identifier(&mut self) -> Result<String> {
        match self.next_token()? {
            Some(Token::Ident(text)) => {
                if !is_lowercase_identifier(&text) {
                    return Err(Error::ExpectedLowercaseIdentifier {
                        got: format!("Ident({text:?})"),
                    });
                }
                Ok(text)
            }
            other => Err(Error::ExpectedLowercaseIdentifier {
                got: format!("{other:?}"),
            }),
        }
    }
}

// ─── small private helper ─────────────────────────────────────

fn check_bind_name(got: &str, expected_field_name: &str) -> Result<()> {
    if got == expected_field_name {
        Ok(())
    } else {
        Err(Error::WrongBindName {
            expected_field_name: expected_field_name.to_string(),
            got_bind_name: got.to_string(),
        })
    }
}
