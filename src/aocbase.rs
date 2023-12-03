
use thiserror::Error;

use std::num::ParseIntError;
use std::io;

#[derive(Debug, Error)]
pub enum AOCError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Invalid regex operation.")]
    InvalidRegexOperation(),
}

pub type AOCResult<T> = Result<T, AOCError>;

impl From<ParseIntError> for AOCError {
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(format!("{value}"))
    }
}

impl From<io::Error> for AOCError {
    fn from(value: io::Error) -> Self {
        Self::IOError(format!("{value}"))
    }
}
