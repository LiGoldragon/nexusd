use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("i/o: {0}")]
    Io(#[from] std::io::Error),

    #[error("lexer: {0}")]
    Lex(String),

    #[error("expected token {expected}, got {got}")]
    UnexpectedToken { expected: String, got: String },

    #[error("expected end of input, got {got}")]
    TrailingTokens { got: String },

    #[error("expected PascalCase identifier (type or variant name), got {got}")]
    ExpectedPascalIdentifier { got: String },

    #[error("expected lowercase identifier (camelCase or kebab-case), got {got}")]
    ExpectedLowercaseIdentifier { got: String },

    #[error("unknown record kind in query: {kind_name}")]
    UnknownQueryKind { kind_name: String },

    #[error("expected pattern field at this position (one of `_`, `@<field-name>`, or a literal value), got {got}")]
    ExpectedPatternField { got: String },

    #[error(
        "bind name mismatch: position expected `@{expected_field_name}` (the schema field name); got `@{got_bind_name}`. \
         Bind names MUST equal the schema field name at that position — see nexus/spec/grammar.md §Binds."
    )]
    WrongBindName {
        expected_field_name: String,
        got_bind_name: String,
    },

    #[error("expected variant name (one of {expected_variants}), got `{got}`")]
    UnknownRelationKindVariant {
        expected_variants: String,
        got: String,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
