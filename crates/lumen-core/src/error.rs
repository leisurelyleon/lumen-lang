//! Error types for each stage, plus a unifying top-level error.

/// A lexing error with its source position.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("lex error at line {line}, col {col}: {message}")]
pub struct LexError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

/// A parsing error with its source position.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("parse error at line {line}, col {col}: {message}")]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

/// A runtime (evaluation) error.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[error("runtime error: {message}")]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Any error from the full interpret pipeline.
#[derive(Debug, thiserror::Error)]
pub enum LumenError {
    #[error(transparent)]
    Lex(#[from] LexError),
    #[error(transparent)]
    Parse(#[from] ParseError),
    #[error(transparent)]
    Runtime(#[from] RuntimeError),
}
