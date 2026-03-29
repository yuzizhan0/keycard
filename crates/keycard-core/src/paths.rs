//! Default `vault.db` location per `docs/superpowers/specs/2026-03-29-keycard-design.md` §2.

use std::path::PathBuf;

use crate::error::KeycardError;

/// Returns the platform default path to `vault.db` (parent `Keycard/` dir may not exist yet).
///
/// - **macOS:** `~/Library/Application Support/Keycard/vault.db`
/// - **Linux:** `$XDG_DATA_HOME/Keycard/vault.db` or `~/.local/share/Keycard/vault.db`
/// - **Windows:** `%LOCALAPPDATA%\\Keycard\\vault.db`
pub fn vault_db_path() -> Result<PathBuf, KeycardError> {
    let base = if cfg!(windows) {
        dirs::data_local_dir()
    } else {
        dirs::data_dir()
    }
    .ok_or(KeycardError::DataDirNotFound)?;

    Ok(base.join("Keycard").join("vault.db"))
}

#[cfg(test)]
mod tests {
    use super::vault_db_path;

    #[test]
    fn vault_db_path_ends_with_keycard_vault_db() {
        let p = vault_db_path().expect("path");
        assert!(
            p.to_string_lossy().contains("Keycard"),
            "expected Keycard in path, got {}",
            p.display()
        );
        assert_eq!(p.file_name().unwrap(), "vault.db");
    }
}
