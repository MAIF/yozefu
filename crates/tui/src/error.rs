use lib::Error;
use std::{
    fmt::{self, Display, Formatter},
    num::TryFromIntError,
};
use tokio::sync::mpsc::error::SendError;

/// Wrapper of around the `lib::Error` struct.
//#[derive(Debug)]
pub struct TuiError(Error);

impl<T> From<SendError<T>> for TuiError {
    fn from(e: SendError<T>) -> Self {
        TuiError(Error::Error(e.to_string()))
    }
}

impl Display for TuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

use std::fmt::Debug;

impl Debug for TuiError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for TuiError {
    fn from(e: &str) -> Self {
        TuiError(Error::Error(e.to_string()))
    }
}

impl From<serde_json::Error> for TuiError {
    fn from(e: serde_json::Error) -> Self {
        TuiError(Error::SerdeError(e))
    }
}

impl From<Error> for TuiError {
    fn from(e: Error) -> Self {
        TuiError(e)
    }
}

impl From<TuiError> for Error {
    fn from(val: TuiError) -> Self {
        val.0
    }
}

impl From<std::io::Error> for TuiError {
    fn from(e: std::io::Error) -> Self {
        TuiError(Error::IoError(e))
    }
}

impl From<TryFromIntError> for TuiError {
    fn from(e: TryFromIntError) -> Self {
        TuiError(Error::Error(e.to_string()))
    }
}

impl std::error::Error for TuiError {}
