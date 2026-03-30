//! First-time vault creation and password unlock (`UnlockedVault`).

use std::collections::BTreeMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rand_core::{OsRng, RngCore};
use rusqlite::Connection;
use zeroize::Zeroizing;

use crate::crypto::{derive_dek, derive_master_key, open_secret, seal_secret};
use crate::db::open_vault;
use crate::models::{CliFavoriteMeta, EntryMeta, ProfileMeta};
use crate::session::UnlockedDek;
use crate::KeycardError;

/// `meta.key` for schema version (value: UTF-8 `"1"`).
pub const META_SCHEMA_VERSION: &str = "schema_version";
/// `meta.key` for Argon2 salt (value: random bytes, 16 bytes in v1).
pub const META_KDF_SALT: &str = "kdf_salt";
/// Encrypted sentinel proving the master password (nonce + ciphertext in separate rows).
pub const META_PW_VERIFY_NONCE: &str = "pw_verify_nonce";
pub const META_PW_VERIFY_CIPHER: &str = "pw_verify_cipher";

const KDF_SALT_LEN: usize = 16;
const PW_VERIFY_PLAIN: &[u8] = b"KEYCARD_V1";

fn now_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// `true` if `path` exists and contains `kdf_salt` in `meta`.
pub fn is_vault_initialized(path: &Path) -> Result<bool, KeycardError> {
    if !path.exists() {
        return Ok(false);
    }
    let conn = open_vault(path)?;
    Ok(meta_get(&conn, META_KDF_SALT)?.is_some())
}

/// Create a new vault at `path`: parent directories, DB schema, random KDF salt, and schema version.
///
/// `password` must be non-empty. Stores KDF salt plus an encrypted verifier so [`Vault::unlock`] can reject wrong passwords.
///
/// Returns [`KeycardError::VaultAlreadyInitialized`] if `kdf_salt` is already set.
pub fn init_vault(path: &Path, password: &[u8]) -> Result<(), KeycardError> {
    if password.is_empty() {
        return Err(KeycardError::InvalidPassword);
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut conn = open_vault(path)?;
    if meta_get(&conn, META_KDF_SALT)?.is_some() {
        return Err(KeycardError::VaultAlreadyInitialized);
    }
    let mut salt = [0u8; KDF_SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let master = derive_master_key(password, &salt)?;
    let dek = derive_dek(&master)?;
    let (pw_nonce, pw_ct) = seal_secret(&dek, PW_VERIFY_PLAIN)?;
    let tx = conn.transaction()?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_SCHEMA_VERSION, b"1".as_slice()),
    )?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_KDF_SALT, salt.as_slice()),
    )?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_PW_VERIFY_NONCE, pw_nonce.as_slice()),
    )?;
    tx.execute(
        "INSERT INTO meta (key, value) VALUES (?1, ?2)",
        (META_PW_VERIFY_CIPHER, pw_ct.as_slice()),
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
        match (
            meta_get(&self.conn, META_PW_VERIFY_NONCE)?,
            meta_get(&self.conn, META_PW_VERIFY_CIPHER)?,
        ) {
            (Some(nonce), Some(ct)) => match open_secret(&dek, &nonce, &ct) {
                Ok(p) if p.as_slice() == PW_VERIFY_PLAIN => {}
                Ok(_) => return Err(KeycardError::VaultCorrupt),
                Err(_) => return Err(KeycardError::WrongMasterPassword),
            },
            (None, None) => {
                // Legacy vaults without verifier (pre-v1 migration); accept unlock.
            }
            _ => return Err(KeycardError::VaultCorrupt),
        }
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
    /// Expose the DEK for session persistence (e.g. desktop shell holding `path` + DEK).
    pub fn dek_for_crypto(&self) -> &[u8; 32] {
        &*self.dek
    }

    /// Open the vault file again with a known DEK (e.g. GUI holding only the key material).
    pub fn reopen(path: &Path, dek: &[u8; 32]) -> Result<Self, KeycardError> {
        let conn = open_vault(path)?;
        Ok(Self {
            conn,
            dek: Zeroizing::new(*dek),
        })
    }

    /// Resolve profile by exact `id` or unique `name`.
    pub fn resolve_profile_id(&self, name_or_id: &str) -> Result<String, KeycardError> {
        self.conn
            .query_row(
                "SELECT id FROM profiles WHERE id = ?1 OR name = ?1 LIMIT 1",
                [name_or_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => KeycardError::ProfileNotFound,
                _ => e.into(),
            })
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
        let (nonce, ciphertext) = seal_secret(self.dek_for_crypto(), secret)?;
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
        Ok(open_secret(self.dek_for_crypto(), &nonce, &ciphertext)?)
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
        let (nonce, ciphertext) = seal_secret(self.dek_for_crypto(), secret)?;
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

    /// Read UI/settings key from `app_settings` (plain string value).
    pub fn get_app_setting(&self, key: &str) -> Result<Option<String>, KeycardError> {
        match self.conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn set_app_setting(&mut self, key: &str, value: &str) -> Result<(), KeycardError> {
        self.conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            (key, value),
        )?;
        Ok(())
    }

    /// List saved CLI commands (sorted for stable UI).
    pub fn list_cli_favorites(&self) -> Result<Vec<CliFavoriteMeta>, KeycardError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, profile_id, argv_json, notes FROM cli_favorites ORDER BY sort_order ASC, name ASC, id ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for r in rows {
            let (id, name, profile_id, argv_json, notes) = r?;
            let argv = parse_argv_json(&argv_json)?;
            out.push(CliFavoriteMeta {
                id,
                name,
                profile_id,
                argv,
                notes,
            });
        }
        Ok(out)
    }

    /// Insert a saved command. `name` must be unique. `argv` must be non-empty (program first).
    pub fn add_cli_favorite(
        &mut self,
        id: &str,
        name: &str,
        profile_id: Option<&str>,
        argv: &[String],
        notes: Option<&str>,
    ) -> Result<(), KeycardError> {
        validate_cli_argv(argv)?;
        if let Some(pid) = profile_id {
            self.ensure_profile_exists(pid)?;
        }
        let argv_json = serde_json::to_string(argv).map_err(|_| KeycardError::InvalidCliFavorite)?;
        let created_at = now_millis();
        let next_sort: i64 = self.conn.query_row(
            "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM cli_favorites",
            [],
            |row| row.get(0),
        )?;
        let n = self.conn.execute(
            "INSERT INTO cli_favorites (id, name, profile_id, argv_json, notes, created_at, sort_order) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                id,
                name,
                profile_id,
                argv_json,
                notes,
                created_at,
                next_sort,
            ),
        );
        match n {
            Ok(_) => Ok(()),
            Err(rusqlite::Error::SqliteFailure(e, _))
                if e.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                Err(KeycardError::CliFavoriteAlreadyExists)
            }
            Err(e) => Err(e.into()),
        }
    }

    pub fn update_cli_favorite(
        &mut self,
        id: &str,
        name: &str,
        profile_id: Option<&str>,
        argv: &[String],
        notes: Option<&str>,
    ) -> Result<(), KeycardError> {
        validate_cli_argv(argv)?;
        if let Some(pid) = profile_id {
            self.ensure_profile_exists(pid)?;
        }
        let argv_json = serde_json::to_string(argv).map_err(|_| KeycardError::InvalidCliFavorite)?;
        let n = self
            .conn
            .execute(
                "UPDATE cli_favorites SET name = ?2, profile_id = ?3, argv_json = ?4, notes = ?5 WHERE id = ?1",
                (id, name, profile_id, argv_json, notes),
            )
            .map_err(|e| {
                if matches!(
                    e,
                    rusqlite::Error::SqliteFailure(ref se, _)
                        if se.code == rusqlite::ErrorCode::ConstraintViolation
                ) {
                    KeycardError::CliFavoriteAlreadyExists
                } else {
                    e.into()
                }
            })?;
        if n == 0 {
            return Err(KeycardError::CliFavoriteNotFound);
        }
        Ok(())
    }

    pub fn delete_cli_favorite(&mut self, id: &str) -> Result<(), KeycardError> {
        let n = self
            .conn
            .execute("DELETE FROM cli_favorites WHERE id = ?1", [id])?;
        if n == 0 {
            return Err(KeycardError::CliFavoriteNotFound);
        }
        Ok(())
    }

    /// Resolve saved command by exact `name` (for `keycard saved run <name>`).
    pub fn get_cli_favorite_by_name(
        &self,
        name: &str,
    ) -> Result<(Vec<String>, Option<String>), KeycardError> {
        let row: Result<(String, Option<String>), rusqlite::Error> = self.conn.query_row(
            "SELECT argv_json, profile_id FROM cli_favorites WHERE name = ?1",
            [name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        match row {
            Ok((argv_json, profile_id)) => {
                let argv = parse_argv_json(&argv_json)?;
                validate_cli_argv(&argv)?;
                Ok((argv, profile_id))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(KeycardError::CliFavoriteNotFound),
            Err(e) => Err(e.into()),
        }
    }
}

fn validate_cli_argv(argv: &[String]) -> Result<(), KeycardError> {
    if argv.is_empty() || argv.iter().any(|s| s.is_empty()) {
        return Err(KeycardError::InvalidCliFavorite);
    }
    Ok(())
}

fn parse_argv_json(raw: &str) -> Result<Vec<String>, KeycardError> {
    let v: serde_json::Value =
        serde_json::from_str(raw).map_err(|_| KeycardError::InvalidCliFavorite)?;
    let arr = v
        .as_array()
        .ok_or(KeycardError::InvalidCliFavorite)?;
    let mut out = Vec::with_capacity(arr.len());
    for x in arr {
        let s = x
            .as_str()
            .ok_or(KeycardError::InvalidCliFavorite)?
            .to_string();
        out.push(s);
    }
    Ok(out)
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
        tempdir().expect("tempdir").keep().join("vault.db")
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
        assert!(matches!(
            vault.unlock(b""),
            Err(crate::KeycardError::InvalidPassword)
        ));
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

    #[test]
    fn cli_favorites_roundtrip() {
        let path = temp_vault_path();
        init_vault(&path, b"x").unwrap();
        let mut u = Vault::open(&path).unwrap().unlock(b"x").unwrap();
        u.add_profile("p1", "dev").unwrap();
        let argv = vec!["echo".to_string(), "hi".to_string()];
        u.add_cli_favorite("f1", "hello", Some("p1"), &argv, None)
            .unwrap();
        let list = u.list_cli_favorites().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, "hello");
        assert_eq!(list[0].argv, argv);
        let (gargv, pid) = u.get_cli_favorite_by_name("hello").unwrap();
        assert_eq!(gargv, argv);
        assert_eq!(pid.as_deref(), Some("p1"));
        u.update_cli_favorite("f1", "hello", None, &argv, Some("note"))
            .unwrap();
        assert_eq!(
            u.list_cli_favorites().unwrap()[0].profile_id,
            None::<String>
        );
        u.delete_cli_favorite("f1").unwrap();
        assert!(u.list_cli_favorites().unwrap().is_empty());
    }
}
