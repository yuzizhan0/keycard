//! First-time vault creation and password unlock (`UnlockedVault`).

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rand_core::{OsRng, RngCore};
use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::crypto::{derive_dek, derive_master_key, open_secret, seal_secret};
use crate::db::open_vault;
use crate::models::{EntryMeta, ProfileMeta};
use crate::session::UnlockedDek;
use crate::KeycardError;

/// `meta.key` for schema version (value: UTF-8 `"1"`).
pub const META_SCHEMA_VERSION: &str = "schema_version";
/// `meta.key` for Argon2 salt (value: random bytes, 16 bytes in v1).
pub const META_KDF_SALT: &str = "kdf_salt";

const KDF_SALT_LEN: usize = 16;

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

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

    /// Insert a new encrypted entry (`secret` is sealed with the vault DEK).
    pub fn add_entry(
        &mut self,
        id: &str,
        provider: Option<&str>,
        alias: &str,
        tags: Option<&str>,
        secret: &[u8],
    ) -> Result<(), KeycardError> {
        let (nonce, ciphertext) = seal_secret(self.dek.as_ref(), secret)?;
        let created_at = now_millis();
        self.conn.execute(
            "INSERT INTO entries (id, provider, alias, tags, created_at, nonce, ciphertext) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                id,
                provider,
                alias,
                tags,
                created_at,
                &nonce,
                &ciphertext,
            ),
        )?;
        Ok(())
    }

    /// List entry metadata only (no nonce, ciphertext, or plaintext secret).
    pub fn list_entries_meta(&self) -> Result<Vec<EntryMeta>, KeycardError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider, alias, tags, created_at FROM entries ORDER BY created_at ASC, id ASC",
        )?;
        let mapped = stmt.query_map([], |row| {
            Ok(EntryMeta {
                id: row.get(0)?,
                provider: row.get(1)?,
                alias: row.get(2)?,
                tags: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for row in mapped {
            out.push(row?);
        }
        Ok(out)
    }

    /// Decrypt and return the secret bytes for `id`.
    pub fn get_entry_secret(&self, id: &str) -> Result<Vec<u8>, KeycardError> {
        let (nonce, ciphertext): (Vec<u8>, Vec<u8>) = match self.conn.query_row(
            "SELECT nonce, ciphertext FROM entries WHERE id = ?1",
            [id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ) {
            Ok(v) => v,
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                return Err(KeycardError::EntryNotFound);
            }
            Err(e) => return Err(e.into()),
        };
        Ok(open_secret(self.dek.as_ref(), &nonce, &ciphertext)?)
    }

    pub fn delete_entry(&mut self, id: &str) -> Result<(), KeycardError> {
        let n = self.conn.execute("DELETE FROM entries WHERE id = ?1", [id])?;
        if n == 0 {
            return Err(KeycardError::EntryNotFound);
        }
        Ok(())
    }

    /// Update non-secret fields only.
    pub fn update_entry_meta(
        &mut self,
        id: &str,
        provider: Option<&str>,
        alias: &str,
        tags: Option<&str>,
    ) -> Result<(), KeycardError> {
        let n = self.conn.execute(
            "UPDATE entries SET provider = ?1, alias = ?2, tags = ?3 WHERE id = ?4",
            (provider, alias, tags, id),
        )?;
        if n == 0 {
            return Err(KeycardError::EntryNotFound);
        }
        Ok(())
    }

    /// Replace ciphertext for an existing entry (re-encrypts `secret` with a fresh nonce).
    pub fn set_entry_secret(&mut self, id: &str, secret: &[u8]) -> Result<(), KeycardError> {
        let (nonce, ciphertext) = seal_secret(self.dek.as_ref(), secret)?;
        let n = self.conn.execute(
            "UPDATE entries SET nonce = ?1, ciphertext = ?2 WHERE id = ?3",
            (&nonce, &ciphertext, id),
        )?;
        if n == 0 {
            return Err(KeycardError::EntryNotFound);
        }
        Ok(())
    }

    /// Create a profile (`id` and `name` must both be unused).
    pub fn add_profile(&mut self, id: &str, name: &str) -> Result<(), KeycardError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM profiles WHERE id = ?1 OR name = ?2 LIMIT 1")?;
        if stmt.exists((id, name))? {
            return Err(KeycardError::ProfileAlreadyExists);
        }
        self.conn
            .execute(
                "INSERT INTO profiles (id, name) VALUES (?1, ?2)",
                (id, name),
            )
            .map_err(KeycardError::from)?;
        Ok(())
    }

    pub fn list_profiles(&self) -> Result<Vec<ProfileMeta>, KeycardError> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM profiles ORDER BY name ASC, id ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ProfileMeta {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Map `env_var` → `entry_id` for this profile (upserts on duplicate `env_var`).
    pub fn set_profile_env(
        &mut self,
        profile_id: &str,
        env_var: &str,
        entry_id: &str,
    ) -> Result<(), KeycardError> {
        self.ensure_profile_exists(profile_id)?;
        self.ensure_entry_exists(entry_id)?;
        self.conn.execute(
            "INSERT INTO profile_env (profile_id, env_var, entry_id) VALUES (?1, ?2, ?3)
             ON CONFLICT(profile_id, env_var) DO UPDATE SET entry_id = excluded.entry_id",
            (profile_id, env_var, entry_id),
        )?;
        Ok(())
    }

    /// Environment variable name → entry id for `profile_id` (sorted by `env_var`).
    pub fn profile_env_mappings(
        &self,
        profile_id: &str,
    ) -> Result<BTreeMap<String, String>, KeycardError> {
        self.ensure_profile_exists(profile_id)?;
        let mut stmt = self.conn.prepare(
            "SELECT env_var, entry_id FROM profile_env WHERE profile_id = ?1 ORDER BY env_var ASC",
        )?;
        let rows = stmt.query_map([profile_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut m = BTreeMap::new();
        for r in rows {
            let (k, v) = r?;
            m.insert(k, v);
        }
        Ok(m)
    }

    pub fn delete_profile(&mut self, profile_id: &str) -> Result<(), KeycardError> {
        let n = self
            .conn
            .execute("DELETE FROM profiles WHERE id = ?1", [profile_id])?;
        if n == 0 {
            return Err(KeycardError::ProfileNotFound);
        }
        Ok(())
    }

    fn ensure_profile_exists(&self, profile_id: &str) -> Result<(), KeycardError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM profiles WHERE id = ?1 LIMIT 1")?;
        if !stmt.exists([profile_id])? {
            return Err(KeycardError::ProfileNotFound);
        }
        Ok(())
    }

    fn ensure_entry_exists(&self, entry_id: &str) -> Result<(), KeycardError> {
        let mut stmt = self
            .conn
            .prepare("SELECT 1 FROM entries WHERE id = ?1 LIMIT 1")?;
        if !stmt.exists([entry_id])? {
            return Err(KeycardError::EntryNotFound);
        }
        Ok(())
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
            .add_entry(&id, Some("openai"), "default", None, secret)
            .expect("add");
        let out = unlocked.get_entry_secret(&id).expect("get secret");
        assert_eq!(out, secret);
    }

    #[test]
    fn list_entries_meta_contains_no_secret_substring() {
        let path = temp_vault_path();
        let password = b"p";
        init_vault(&path, password).unwrap();
        let vault = Vault::open(&path).unwrap();
        let mut u = vault.unlock(password).unwrap();
        let id = Uuid::new_v4().to_string();
        let distinctive = b"XYZZY_PLAINTEXT_SECRET_99";
        u.add_entry(&id, None, "a1", Some("t1"), distinctive)
            .unwrap();
        let metas = u.list_entries_meta().unwrap();
        assert_eq!(metas.len(), 1);
        let dump = format!("{metas:?}");
        assert!(
            !dump.contains("XYZZY"),
            "list must not leak secret: {dump}"
        );
        assert_eq!(u.get_entry_secret(&id).unwrap(), distinctive);
    }

    #[test]
    fn delete_update_entry() {
        let path = temp_vault_path();
        init_vault(&path, b"x").unwrap();
        let mut u = Vault::open(&path).unwrap().unlock(b"x").unwrap();
        let id = Uuid::new_v4().to_string();
        u.add_entry(&id, Some("p"), "old", None, b"one").unwrap();
        u.update_entry_meta(&id, Some("p2"), "new", Some("tag"))
            .unwrap();
        let m = u.list_entries_meta().unwrap();
        assert_eq!(m[0].alias, "new");
        assert_eq!(m[0].provider.as_deref(), Some("p2"));
        u.set_entry_secret(&id, b"two").unwrap();
        assert_eq!(u.get_entry_secret(&id).unwrap(), b"two");
        u.delete_entry(&id).unwrap();
        assert!(matches!(
            u.get_entry_secret(&id),
            Err(crate::KeycardError::EntryNotFound)
        ));
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

    #[test]
    fn profile_maps_openai_key_to_entry() {
        let path = temp_vault_path();
        init_vault(&path, b"x").unwrap();
        let mut u = Vault::open(&path).unwrap().unlock(b"x").unwrap();
        let entry_id = Uuid::new_v4().to_string();
        u.add_entry(&entry_id, Some("openai"), "default", None, b"sk-test")
            .unwrap();
        u.add_profile("p-dev", "dev").unwrap();
        u.set_profile_env("p-dev", "OPENAI_API_KEY", &entry_id)
            .unwrap();
        let m = u.profile_env_mappings("p-dev").unwrap();
        assert_eq!(m.get("OPENAI_API_KEY").map(String::as_str), Some(entry_id.as_str()));
        assert_eq!(m.len(), 1);
        // Upsert same env to same entry id (idempotent)
        u.set_profile_env("p-dev", "OPENAI_API_KEY", &entry_id)
            .unwrap();
        assert_eq!(u.profile_env_mappings("p-dev").unwrap().len(), 1);
    }

    #[test]
    fn set_profile_env_requires_profile_and_entry() {
        let path = temp_vault_path();
        init_vault(&path, b"x").unwrap();
        let mut u = Vault::open(&path).unwrap().unlock(b"x").unwrap();
        u.add_profile("p1", "n1").unwrap();
        let eid = Uuid::new_v4().to_string();
        assert!(matches!(
            u.set_profile_env("p1", "K", &eid),
            Err(crate::KeycardError::EntryNotFound)
        ));
    }
}
