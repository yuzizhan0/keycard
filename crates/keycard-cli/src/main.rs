//! Keycard CLI — see README for `KEYCARD_ALLOW_ENV_PASSWORD` (testing only).

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use keycard_core::{
    init_vault, vault_db_path, KeycardError, UnlockedVault, Vault,
};

#[derive(Parser)]
#[command(name = "keycard", version, about = "Local API key vault (CLI)")]
struct Cli {
    /// Vault database path (default: platform Keycard/vault.db).
    #[arg(long, global = true, value_name = "PATH")]
    vault: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new vault (fails if one already exists at the path).
    Init,
    /// Print POSIX `export VAR='…'` lines for a profile (stdout only).
    Env {
        #[arg(long, short = 'p', value_name = "NAME_OR_ID")]
        profile: String,
    },
    /// Run a command with profile env vars merged in (inherits current environment).
    Run {
        #[arg(long, short = 'p', value_name = "NAME_OR_ID")]
        profile: String,
        #[arg(
            required = true,
            trailing_var_arg = true,
            allow_hyphen_values = true,
            num_args = 1..
        )]
        cmd: Vec<OsString>,
    },
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("keycard: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> Result<(), String> {
    let cli = Cli::parse();
    let vault_path = cli
        .vault
        .clone()
        .or_else(|| vault_db_path().ok())
        .ok_or_else(|| "could not determine vault path; use --vault".to_string())?;

    match cli.command {
        Commands::Init => cmd_init(&vault_path),
        Commands::Env { profile } => cmd_env(&vault_path, &profile),
        Commands::Run { profile, cmd } => cmd_run(&vault_path, &profile, cmd),
    }
}

fn cmd_init(path: &PathBuf) -> Result<(), String> {
    let p1 = read_password("New master password: ")?;
    let p2 = read_password("Confirm master password: ")?;
    if p1 != p2 {
        return Err("passwords do not match".into());
    }
    init_vault(path, p1.as_bytes()).map_err(fmt_keycard_err)?;
    eprintln!("Vault created at {}.", path.display());
    Ok(())
}

fn cmd_env(path: &PathBuf, profile: &str) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let v = open_unlocked(path, pw.as_bytes())?;
    print_exports(&v, profile)?;
    Ok(())
}

fn cmd_run(path: &PathBuf, profile: &str, cmd: Vec<OsString>) -> Result<(), String> {
    if cmd.is_empty() {
        return Err("missing command after profile flags".into());
    }
    let pw = read_password("Master password: ")?;
    let v = open_unlocked(path, pw.as_bytes())?;
    let map = env_map(&v, profile)?;
    drop(v);

    let mut it = cmd.into_iter();
    let program = it.next().expect("non-empty");
    let mut child = std::process::Command::new(program);
    child.args(it);
    child.envs(std::env::vars());
    for (k, val) in map {
        child.env(k, val);
    }
    let status = child.status().map_err(|e| e.to_string())?;
    std::process::exit(status.code().unwrap_or(1));
}

fn open_unlocked(path: &PathBuf, password: &[u8]) -> Result<UnlockedVault, String> {
    Vault::open(path)
        .map_err(fmt_keycard_err)?
        .unlock(password)
        .map_err(fmt_keycard_err)
}

fn print_exports(v: &UnlockedVault, profile: &str) -> Result<(), String> {
    let map = env_map(v, profile)?;
    for (var, val) in map {
        println!("export {}={}", var, shell_single_quoted(&val));
    }
    Ok(())
}

fn env_map(v: &UnlockedVault, profile: &str) -> Result<std::collections::BTreeMap<String, String>, String> {
    let pid = v.resolve_profile_id(profile).map_err(fmt_keycard_err)?;
    let bindings = v.profile_env_mappings(&pid).map_err(fmt_keycard_err)?;
    let mut out = std::collections::BTreeMap::new();
    for (env_var, entry_id) in bindings {
        let secret = v.get_entry_secret(&entry_id).map_err(fmt_keycard_err)?;
        let s = String::from_utf8(secret).map_err(|_| "entry secret is not valid UTF-8".to_string())?;
        out.insert(env_var, s);
    }
    Ok(out)
}

/// POSIX-safe single-quoted string for use after `export VAR=`.
fn shell_single_quoted(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\"'\"'"))
}

fn read_password(prompt: &str) -> Result<String, String> {
    if std::env::var("KEYCARD_ALLOW_ENV_PASSWORD").ok().as_deref() == Some("1") {
        if let Ok(p) = std::env::var("KEYCARD_MASTER_PASSWORD") {
            if !p.is_empty() {
                return Ok(p);
            }
        }
    }
    rpassword::prompt_password(prompt).map_err(|e| e.to_string())
}

fn fmt_keycard_err(e: KeycardError) -> String {
    e.to_string()
}
