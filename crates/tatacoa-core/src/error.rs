use std::fmt::{self, Display, Formatter};
use std::io;

#[derive(Debug)]
pub enum Error {
    Io {
        operation: &'static str,
        source: io::Error,
    },
    Json(serde_json::Error),
    InvalidId {
        kind: &'static str,
        value: String,
    },
    InvalidPath(String),
    InvalidManifest(String),
    Conflict(String),
    Cryptography(&'static str),
    Execution(String),
}

impl Error {
    pub fn io(operation: &'static str, source: io::Error) -> Self {
        Self::Io { operation, source }
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { operation, source } => write!(formatter, "{operation}: {source}"),
            Self::Json(source) => write!(formatter, "invalid JSON: {source}"),
            Self::InvalidId { kind, value } => write!(formatter, "invalid {kind} ID: {value}"),
            Self::InvalidPath(message) => write!(formatter, "unsafe path: {message}"),
            Self::InvalidManifest(message) => write!(formatter, "invalid manifest: {message}"),
            Self::Conflict(message) => write!(formatter, "conflict: {message}"),
            Self::Cryptography(message) => {
                write!(formatter, "cryptographic operation failed: {message}")
            }
            Self::Execution(message) => write!(formatter, "execution failed: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::Json(source) => Some(source),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(source: serde_json::Error) -> Self {
        Self::Json(source)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
