//! Minimal errors for early tasks; extended in later tasks.

use std::io;

#[derive(Debug)]
pub enum KeycardError {
    Io(io::Error),
    DataDirNotFound,
    Sqlite(rusqlite::Error),
    Crypto(crate::crypto::CryptoError),
    /// `init_vault` was called but `kdf_salt` is already present.
    VaultAlreadyInitialized,
    /// `unlock` called on a vault with no `kdf_salt` (run `init_vault` first).
    VaultNotInitialized,
    /// Empty master password (not allowed).
    InvalidPassword,
    /// No row with the given entry id.
    EntryNotFound,
    /// No profile with the given id.
    ProfileNotFound,
    /// Profile id or display name already exists.
    ProfileAlreadyExists,
}

impl std::fmt::Display for KeycardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeycardError::Io(e) => write!(f, "{e}"),
            KeycardError::DataDirNotFound => write!(f, "platform data directory not found"),
            KeycardError::Sqlite(e) => write!(f, "sqlite: {e}"),
            KeycardError::Crypto(e) => write!(f, "crypto: {e}"),
            KeycardError::VaultAlreadyInitialized => write!(f, "vault already initialized"),
            KeycardError::VaultNotInitialized => write!(f, "vault not initialized"),
            KeycardError::InvalidPassword => write!(f, "invalid password"),
            KeycardError::EntryNotFound => write!(f, "entry not found"),
            KeycardError::ProfileNotFound => write!(f, "profile not found"),
            KeycardError::ProfileAlreadyExists => write!(f, "profile already exists"),
        }
    }
}

impl std::error::Error for KeycardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            KeycardError::Io(e) => Some(e),
            KeycardError::Sqlite(e) => Some(e),
            KeycardError::DataDirNotFound => None,
            KeycardError::Crypto(e) => Some(e),
            KeycardError::VaultAlreadyInitialized
            | KeycardError::VaultNotInitialized
            | KeycardError::InvalidPassword
            | KeycardError::EntryNotFound
            | KeycardError::ProfileNotFound
            | KeycardError::ProfileAlreadyExists => None,
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

impl From<crate::crypto::CryptoError> for KeycardError {
    fn from(value: crate::crypto::CryptoError) -> Self {
        KeycardError::Crypto(value)
    }
}
