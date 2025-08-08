use std::{error::Error as StdError, fmt};

use super::structural_types::BalancingError;
use crate::lexer;

pub type Result<T> = std::result::Result<T, Error>;

#[non_exhaustive]
#[derive(Debug, PartialEq)]
pub enum Error {
    Char(CharError),
    NotClosable,
    Corrupted,
}

#[derive(Debug, PartialEq)]
pub struct CharError(pub(crate) lexer::JSONParseError);

impl fmt::Display for CharError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid char for current state: {:?}", self.0)
    }
}
impl StdError for CharError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Char(e) => e.fmt(f),
            Error::NotClosable => write!(f, "not closable yet"),
            Error::Corrupted => write!(f, "corrupted stream"),
        }
    }
}
impl StdError for Error {}

impl From<lexer::JSONParseError> for CharError {
    fn from(e: lexer::JSONParseError) -> Self {
        CharError(e)
    }
}
impl From<lexer::JSONParseError> for Error {
    fn from(e: lexer::JSONParseError) -> Self {
        Error::Char(CharError(e))
    }
}
impl From<BalancingError> for Error {
    fn from(e: BalancingError) -> Self {
        match e {
            BalancingError::NotClosable => Error::NotClosable,
            BalancingError::Corrupted => Error::Corrupted,
        }
    }
}
