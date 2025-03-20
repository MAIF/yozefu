//! Error definitions and their implementations of the `From` trait.
use rdkafka::error::KafkaError;
use std::fmt::Display;
use std::str::Utf8Error;
use std::{
    fmt::{self, Formatter},
    num::TryFromIntError,
};

/// All kinds of errors that may occur
#[derive(Debug)]
pub enum Error {
    Error(String),
    KafkaError(KafkaError),
    IoError(std::io::Error),
    SerdeError(serde_json::Error),
    ThemeError(String),
    Search(SearchError),
    SchemaRegistry(String),
    Tokio(String),
}

#[derive(Debug)]
pub enum SearchError {
    Parse(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::IoError(e) => write!(f, "IO Error: {}", e),
            Error::SerdeError(e) => write!(f, "Serde Error: {}", e),
            Error::ThemeError(e) => write!(f, "Theme Error: {}", e),
            Error::KafkaError(e) => write!(f, "Kafka Error: {}", e),
            Error::Error(e) => write!(f, "{}", e),
            Error::Tokio(e) => write!(f, "Tokio Error: {}", e),
            Error::Search(e) => write!(f, "{}", e),
            Error::SchemaRegistry(e) => write!(f, "Schema registry Error: {}", e),
        }
    }
}

impl Display for SearchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::Parse(e) => write!(f, "Cannot parse the search query at '{}'", e),
        }
    }
}

impl From<std::fmt::Error> for Error {
    fn from(e: std::fmt::Error) -> Self {
        Error::Error(e.to_string())
    }
}

impl From<KafkaError> for Error {
    fn from(e: KafkaError) -> Self {
        Error::KafkaError(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::SerdeError(e)
    }
}

impl From<&str> for Error {
    fn from(e: &str) -> Self {
        Error::Error(e.to_string())
    }
}

impl From<TryFromIntError> for Error {
    fn from(e: TryFromIntError) -> Self {
        Error::Error(e.to_string())
    }
}

impl From<strum::ParseError> for Error {
    fn from(e: strum::ParseError) -> Self {
        Error::Error(e.to_string())
    }
}

impl From<Utf8Error> for Error {
    fn from(e: Utf8Error) -> Self {
        Error::Error(e.to_string())
    }
}
