use serde::Serialize;
use std::fmt;

/// Error returned to the UI: a stable `code` (translated by the UI) plus a
/// developer-oriented message.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Error {
    pub code: String,
    pub message: String,
}

impl Error {
    pub fn new(code: &str, message: impl Into<String>) -> Error {
        Error { code: code.into(), message: message.into() }
    }
    pub fn not_found(what: &str, id: u64) -> Error {
        Error::new("not_found", format!("{what} #{id} does not exist"))
    }
    pub fn invalid(message: impl Into<String>) -> Error {
        Error::new("invalid", message)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for Error {}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::new("invalid", e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
