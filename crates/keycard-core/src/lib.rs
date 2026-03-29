//! Keycard core library: vault storage, crypto, and platform paths.

pub mod error;
pub mod paths;

pub use error::KeycardError;
pub use paths::vault_db_path;
