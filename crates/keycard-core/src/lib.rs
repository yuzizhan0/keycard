//! Keycard core library: vault storage, crypto, and platform paths.

pub mod crypto;
pub mod db;
pub mod error;
pub mod models;
pub mod paths;
pub mod session;
pub mod vault;

pub use crypto::{derive_dek, derive_master_key, open_secret, seal_secret, CryptoError};
pub use db::{open_vault, BUSY_TIMEOUT_MS};
pub use error::KeycardError;
pub use models::{CliFavoriteMeta, EntryMeta, ProfileMeta};
pub use paths::{keycard_data_dir, pending_cli_snippet_path, vault_db_path};
pub use session::UnlockedDek;
pub use vault::{
    init_vault, is_vault_initialized, UnlockedVault, Vault, META_KDF_SALT, META_SCHEMA_VERSION,
};
