//! First-time vault creation and password unlock (`UnlockedVault`).

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rand_core::{OsRng, RngCore};
use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::crypto::{derive_dek, derive_master_key, open_secret, seal_secret};
use crate::db::open_vault;
use crate::session::UnlockedDek;
use crate::KeycardError;

/// `meta.key` for schema version (value: UTF-8 `"1"`).
pub const META_SCHEMA_VERSION: &str = "schema_version";
/// `meta.key` for Argon2 salt (value: random bytes, 16 bytes in v1).
pub const META_KDF_SALT: &str = "kdf_salt";

const KDF_SALT_LEN: usize = 16;

/// Create a new vault at `path`: parent directories, DB schema, random KDF salt, and schema version.
///
/// `password` must be non-empty (reserved for future verification flows; v1 only stores the salt).
///
/// Returns [`KeycardError::VaultAlreadyInitialized`] if `kdf_salt` is already set.
pub fn init_vault(path: &Path, password: &[u8]) -> Result<(), KeycardError> {
    if password.is_empty() {
        return Err(KeycardError::InvalidPassword);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = open_vault(path)?;
    if meta_get(&conn, META_KDF_SALT)?.is_some() {
        return Err(KeycardError::VaultAlreadyInitialized);
    }
    let mut salt = [0u8; KDF_SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_SCHEMA_VERSION, b"1".as_slice()),
    )?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_KDF_SALT, salt.as_slice()),
    )?;
    tx.commit()?;
    Ok(())
}

/// Handle to an on-disk vault (opened connection, schema applied).
pub struct Vault {
    conn: Connection,
}

impl Vault {
    pub fn open(path: &Path) -> Result<Self, KeycardError> {
        Ok(Self {
            conn: open_vault(path)?,
        })
    }

    /// Unlocks with the master password and returns a handle that holds the DEK in memory ([`UnlockedDek`]).
    pub fn unlock(self, password: &[u8]) -> Result<UnlockedVault, KeycardError> {
        if password.is_empty() {
            return Err(KeycardError::InvalidPassword);
        }
        let salt = meta_get(&self.conn, META_KDF_SALT)?.ok_or(KeycardError::VaultNotInitialized)?;
        let master = derive_master_key(password, &salt)?;
        let dek = derive_dek(&master)?;
        Ok(UnlockedVault {
            conn: self.conn,
            dek: Zeroizing::new(dek),
        })
    }
}

/// Unlocked vault: SQLite connection + in-memory DEK (zeroed on drop).
pub struct UnlockedVault {
    conn: Connection,
    dek: UnlockedDek,
}

impl UnlockedVault {
    pub(crate) fn dek_for_crypto(&self) -> &[u8; 32] {
        self.dek.as_ref()
    }

    /// Encrypt `secret` and insert a row into `entries` (Task 4 / tests; public API for higher layers later).
    pub fn add_encrypted_entry(
        &mut self,
        id: &str,
        provider: Option<&str>,
        alias: &str,
        secret: &[u8],
    ) -> Result<(), KeycardError> {
        let (nonce, ciphertext) = seal_secret(self.dek.as_ref(), secret)?;
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        self.conn.execute(
            "INSERT INTO entries (id, provider, alias, tags, created_at, nonce, ciphertext) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                id,
                provider,
                alias,
                Option::<&str>::None,
                created_at,
                &nonce,
                &ciphertext,
            ),
        )?;
        Ok(())
    }

    /// Read ciphertext for an entry (used in tests).
    pub fn fetch_entry_ciphertext(
        &self,
        id: &str,
    ) -> Result<(Vec<u8>, Vec<u8>), KeycardError> {
        let (nonce, ct): (Vec<u8>, Vec<u8>) = self.conn.query_row(
            "SELECT nonce, ciphertext FROM entries WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok((nonce, ct))
    }
}

fn meta_get(conn: &Connection, key: &str) -> Result<Option<Vec<u8>>, KeycardError> {
    let mut stmt = conn.prepare("SELECT value FROM meta WHERE key = ?1")?;
    let mut rows = stmt.query([key])?;
    match rows.next()? {
        Some(row) => Ok(Some(row.get(0)?)),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::{init_vault, Vault};
    use crate::crypto::open_secret;
    use std::path::PathBuf;

    use tempfile::tempdir;
    use uuid::Uuid;

    fn temp_vault_path() -> PathBuf {
        tempdir().expect("tempdir").into_path().join("vault.db")
    }

    #[test]
    fn init_unlock_seal_roundtrip() {
        let path = temp_vault_path();
        let password = b"correct-horse-battery-staple";
        init_vault(&path, password).expect("init");
        let vault = Vault::open(&path).expect("open");
        let mut unlocked = vault.unlock(password).expect("unlock");
        let id = Uuid::new_v4().to_string();
        let secret = b"sk-dummy-secret-for-task4";
        unlocked
            .add_encrypted_entry(&id, Some("openai"), "default", secret)
            .expect("add");
        let (nonce, ct) = unlocked.fetch_entry_ciphertext(&id).expect("fetch");
        let out = open_secret(unlocked.dek_for_crypto(), &nonce, &ct).expect("open_secret");
        assert_eq!(out, secret);
    }

    #[test]
    fn init_twice_errors() {
        let path = temp_vault_path();
        init_vault(&path, b"p1").unwrap();
        let e = init_vault(&path, b"p2").unwrap_err();
        match e {
            crate::KeycardError::VaultAlreadyInitialized => {}
            other => panic!("expected VaultAlreadyInitialized, got {other:?}"),
        }
    }

    #[test]
    fn unlock_empty_password_errors() {
        let path = temp_vault_path();
        init_vault(&path, b"ok").unwrap();
        let vault = Vault::open(&path).unwrap();
        let e = vault.unlock(b"").unwrap_err();
        match e {
            crate::KeycardError::InvalidPassword => {}
            other => panic!("expected InvalidPassword, got {other:?}"),
        }
    }
}
