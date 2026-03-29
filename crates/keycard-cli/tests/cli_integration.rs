//! Integration tests; use `KEYCARD_ALLOW_ENV_PASSWORD=1` + `KEYCARD_MASTER_PASSWORD` only in CI/tests.

use assert_cmd::Command;
use keycard_core::{init_vault, Vault};
use tempfile::tempdir;

fn bin() -> Command {
    Command::cargo_bin("keycard").expect("keycard binary")
}

#[test]
fn env_prints_export_for_profile() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("vault.db");
    init_vault(&db, b"masterpass").unwrap();
    let mut u = Vault::open(&db).unwrap().unlock(b"masterpass").unwrap();
    u.add_entry("e-openai", Some("openai"), "k1", None, b"sk-testvalue-abc")
        .unwrap();
    u.add_profile("prof-a", "dev").unwrap();
    u.set_profile_env("prof-a", "OPENAI_API_KEY", "e-openai").unwrap();
    drop(u);

    let out = bin()
        .env("KEYCARD_ALLOW_ENV_PASSWORD", "1")
        .env("KEYCARD_MASTER_PASSWORD", "masterpass")
        .args([
            "--vault",
            db.to_str().unwrap(),
            "env",
            "--profile",
            "dev",
        ])
        .output()
        .expect("run");

    assert!(out.status.success(), "stderr={}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("export OPENAI_API_KEY="));
    assert!(stdout.contains("sk-testvalue-abc"));
}

#[test]
fn wrong_password_stderr_must_not_leak_secret() {
    let dir = tempdir().unwrap();
    let db = dir.path().join("vault.db");
    init_vault(&db, b"right-pass").unwrap();
    let mut u = Vault::open(&db).unwrap().unlock(b"right-pass").unwrap();
    u.add_entry("e1", None, "x", None, b"sk-fake-UNIQUE-LEAK-TEST").unwrap();
    u.add_profile("p1", "p").unwrap();
    u.set_profile_env("p1", "K", "e1").unwrap();
    drop(u);

    let out = bin()
        .env("KEYCARD_ALLOW_ENV_PASSWORD", "1")
        .env("KEYCARD_MASTER_PASSWORD", "wrong-pass")
        .args(["--vault", db.to_str().unwrap(), "env", "--profile", "p"])
        .output()
        .expect("run");

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains("sk-fake-UNIQUE-LEAK-TEST"),
        "stderr leaked secret: {stderr}"
    );
}
