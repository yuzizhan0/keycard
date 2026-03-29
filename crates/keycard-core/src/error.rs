//! Minimal errors for early tasks; extended in later tasks.

use std::io;

#[derive(Debug)]
pub enum KeycardError {
    Io(io::Error),
    DataDirNotFound,
    Sqlite(rusqlite::Error),
}

impl std::fmt::Display for KeycardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeycardError::Io(e) => write!(f, "{e}"),
            KeycardError::DataDirNotFound => write!(f, "platform data directory not found"),
            KeycardError::Sqlite(e) => write!(f, "sqlite: {e}"),
        }
    }
}

impl std::error::Error for KeycardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KeycardError::Io(e) => Some(e),
            KeycardError::Sqlite(e) => Some(e),
            KeycardError::DataDirNotFound => None,
        }
    }
}

impl From<io::Error> for KeycardError {
    fn from(value: io::Error) -> Self {
        KeycardError::Io(value)
    }
}

impl From<rusqlite::Error> for KeycardError {
    fn from(value: rusqlite::Error) -> Self {
        KeycardError::Sqlite(value)
    }
}
