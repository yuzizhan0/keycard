//! SQLite vault file: WAL, foreign keys, schema v1.

use std::path::Path;
use std::time::Duration;

use rusqlite::Connection;

use crate::KeycardError;

/// Busy handler timeout when the database is locked (per design spec §2).
pub const BUSY_TIMEOUT_MS: u64 = 5000;

/// Open (or create) the vault database at `path`, apply pragmas and schema v1.
///
/// - `PRAGMA journal_mode=WAL`
/// - `PRAGMA foreign_keys=ON`
/// - `busy_timeout` = [`BUSY_TIMEOUT_MS`]
pub fn open_vault(path: &Path) -> Result<Connection, KeycardError> {
    let conn = Connection::open(path)?;
    conn.busy_timeout(Duration::from_millis(BUSY_TIMEOUT_MS))?;
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        ",
    )?;
    apply_schema_v1(&conn)?;
    Ok(conn)
}

fn apply_schema_v1(conn: &Connection) -> Result<(), KeycardError> {
    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS meta (
            key TEXT PRIMARY KEY NOT NULL,
            value BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS entries (
            id TEXT PRIMARY KEY NOT NULL,
            provider TEXT,
            alias TEXT NOT NULL,
            tags TEXT,
            created_at INTEGER NOT NULL,
            nonce BLOB NOT NULL,
            ciphertext BLOB NOT NULL
        );

        CREATE TABLE IF NOT EXISTS profiles (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL UNIQUE
        );

        CREATE TABLE IF NOT EXISTS profile_env (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            profile_id TEXT NOT NULL,
            env_var TEXT NOT NULL,
            entry_id TEXT NOT NULL,
            FOREIGN KEY (profile_id) REFERENCES profiles(id) ON DELETE CASCADE,
            FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE,
            UNIQUE(profile_id, env_var)
        );

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );
        ",
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{open_vault, BUSY_TIMEOUT_MS};
    use std::path::PathBuf;

    use tempfile::tempdir;

    fn temp_vault_path() -> PathBuf {
        tempdir().expect("tempdir").into_path().join("vault.db")
    }

    #[test]
    fn open_vault_sets_wal_and_foreign_keys() {
        let path = temp_vault_path();
        let conn = open_vault(&path).expect("open_vault");
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .expect("journal_mode");
        assert_eq!(
            journal_mode.to_lowercase(),
            "wal",
            "expected WAL, got {journal_mode}"
        );
        let fk: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign_keys");
        assert_eq!(fk, 1, "foreign_keys should be ON");
    }

    #[test]
    fn profile_env_rejects_invalid_entry_id() {
        let path = temp_vault_path();
        let conn = open_vault(&path).expect("open_vault");
        conn.execute("INSERT INTO profiles (id, name) VALUES ('p1', 'dev')", [])
            .expect("insert profile");
        let err = conn.execute(
            "INSERT INTO profile_env (profile_id, env_var, entry_id) VALUES ('p1', 'OPENAI_API_KEY', 'no-such-entry')",
            [],
        );
        assert!(
            err.is_err(),
            "expected FK violation for missing entry_id, got {err:?}"
        );
    }

    #[test]
    fn busy_timeout_is_configured() {
        let path = temp_vault_path();
        let conn = open_vault(&path).expect("open_vault");
        let ms: i64 = conn
            .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
            .expect("busy_timeout");
        assert_eq!(ms, BUSY_TIMEOUT_MS as i64);
    }
}
