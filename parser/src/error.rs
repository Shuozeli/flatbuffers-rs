#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid UTF-8 in source at byte offset {0}")]
    InvalidUtf8(usize),

    #[error("invalid string literal")]
    InvalidString,

    #[error("unrecognized base type: {0}")]
    UnknownBaseType(String),

    #[error("invalid integer literal: {value} ({reason})")]
    InvalidInteger { value: String, reason: String },

    #[error("invalid float literal: {value} ({reason})")]
    InvalidFloat { value: String, reason: String },

    #[error("unexpected content '{found}': {context}")]
    UnexpectedContent { found: String, context: String },

    #[error("invalid escape sequence: {0}")]
    InvalidEscape(String),

    #[error("syntax error at line {line}, column {column}: {context}")]
    SyntaxError {
        line: usize,
        column: usize,
        context: String,
    },

    #[error("parse timeout exceeded (input may be too large or pathological)")]
    ParseTimeout,

    #[error("input too large: {size} bytes exceeds maximum of {max} bytes")]
    InputTooLarge { size: usize, max: usize },
}

pub type Result<T> = std::result::Result<T, ParseError>;
