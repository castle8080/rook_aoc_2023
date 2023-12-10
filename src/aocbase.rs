
use thiserror::Error;

use std::num::{ParseIntError, TryFromIntError};
use std::io;
use std::string::FromUtf8Error;

use regex;

#[derive(Debug, Error)]
pub enum AOCError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IOError(String),

    #[error("Invalid regex use: {0}")]
    InvalidRegexOperation(String),

    #[error("Problem processing error: {0}")]
    ProcessingError(String)
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

impl From<regex::Error> for AOCError {
    fn from(value: regex::Error) -> Self {
        Self::InvalidRegexOperation(value.to_string())
    }
}

impl From<FromUtf8Error> for AOCError {
    fn from(value: FromUtf8Error) -> Self {
        Self::ParseError(value.to_string())
    }
}

impl From<TryFromIntError> for AOCError {
    fn from(value: TryFromIntError) -> Self {
        Self::ParseError(value.to_string())
    }
}

impl From<csv::Error> for AOCError {
    fn from(value: csv::Error) -> Self {
        Self::IOError(value.to_string())
    }
}