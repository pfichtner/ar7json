use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
pub enum Ar7Error {
    #[error("expected {expected:?}; found `{found}`")]
    #[diagnostic(code(ar7::unexpected_token))]
    UnexpectedToken {
        expected: Vec<String>,
        found: String,
        #[source_code]
        source_code: String,
        #[label("found here")]
        span: SourceSpan,
    },

    #[error("unexpected end of file; expected {expected:?}")]
    #[diagnostic(code(ar7::unexpected_eof))]
    UnexpectedEof {
        expected: Vec<String>,
        #[source_code]
        source_code: String,
        #[label("unexpected EOF")]
        span: SourceSpan,
    },

    #[error("invalid number: {text}")]
    #[diagnostic(code(ar7::invalid_number))]
    InvalidNumber {
        text: String,
        #[source_code]
        source_code: String,
        #[label("invalid number")]
        span: SourceSpan,
    },

    #[error("invalid JSON: {message}")]
    #[diagnostic(code(ar7::invalid_json))]
    InvalidJson { message: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("{message}")]
    #[diagnostic(code(ar7::general))]
    General { message: String },
}

impl Ar7Error {
    pub fn unexpected_token(
        expected: Vec<&str>,
        found: &str,
        source_code: String,
        span: SourceSpan,
    ) -> Self {
        Ar7Error::UnexpectedToken {
            expected: expected.into_iter().map(String::from).collect(),
            found: found.to_string(),
            source_code,
            span,
        }
    }

    pub fn unexpected_eof(expected: Vec<&str>, source_code: String, span: SourceSpan) -> Self {
        Ar7Error::UnexpectedEof {
            expected: expected.into_iter().map(String::from).collect(),
            source_code,
            span,
        }
    }

    pub fn invalid_json(message: &str) -> Self {
        Ar7Error::InvalidJson {
            message: message.to_string(),
        }
    }

    pub fn general(message: &str) -> Self {
        Ar7Error::General {
            message: message.to_string(),
        }
    }
}
