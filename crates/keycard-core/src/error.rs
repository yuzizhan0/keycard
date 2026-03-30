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
    /// Master password incorrect (unlock / verifier mismatch).
    WrongMasterPassword,
    /// Vault metadata inconsistent or tampered.
    VaultCorrupt,
    /// Saved CLI command name not found.
    CliFavoriteNotFound,
    /// Saved CLI command name already exists.
    CliFavoriteAlreadyExists,
    /// Invalid argv JSON or empty command.
    InvalidCliFavorite,
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
            KeycardError::WrongMasterPassword => write!(f, "incorrect master password"),
            KeycardError::VaultCorrupt => write!(f, "vault data is corrupt or incompatible"),
            KeycardError::CliFavoriteNotFound => write!(f, "saved CLI command not found"),
            KeycardError::CliFavoriteAlreadyExists => write!(f, "saved CLI command name already exists"),
            KeycardError::InvalidCliFavorite => write!(f, "invalid saved CLI command (argv)"),
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
            | KeycardError::ProfileAlreadyExists
            | KeycardError::WrongMasterPassword
            | KeycardError::VaultCorrupt
            | KeycardError::CliFavoriteNotFound
            | KeycardError::CliFavoriteAlreadyExists
            | KeycardError::InvalidCliFavorite => None,
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
