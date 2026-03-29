//! Keycard core library: vault storage, crypto, and platform paths.

pub mod crypto;
pub mod error;
pub mod paths;

pub use crypto::{derive_dek, derive_master_key, open_secret, seal_secret, CryptoError};
pub use error::KeycardError;
pub use paths::vault_db_path;
