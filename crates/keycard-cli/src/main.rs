//! Keycard CLI — see README for `KEYCARD_ALLOW_ENV_PASSWORD` (testing only).

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use keycard_core::{
    init_vault, vault_db_path, KeycardError, UnlockedVault, Vault,
};
use uuid::Uuid;

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
    /// Manage CLI projects (groups of saved commands).
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
    /// List or manage saved commands, or run one.
    Saved {
        #[command(subcommand)]
        command: SavedCommands,
    },
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// List all CLI projects.
    List,
    /// Create a new CLI project.
    Add {
        /// Display name for the project.
        name: String,
    },
    /// Delete a project and all its saved commands.
    Delete {
        /// Project display name or id.
        name: String,
    },
}

#[derive(Subcommand)]
enum SavedCommands {
    /// Print saved commands. Tab-separated: project, name, profile, argv (JSON).
    List {
        /// Filter by project name or id.
        #[arg(long, short = 'p', value_name = "PROJECT")]
        project: Option<String>,
    },
    /// Add a new saved command.
    Add {
        /// Command name (unique within the project).
        name: String,
        /// Project name or id (default: "General").
        #[arg(long, value_name = "PROJECT")]
        project: Option<String>,
        /// Profile to load env vars from.
        #[arg(long, value_name = "PROFILE")]
        profile: Option<String>,
        /// Optional notes for this command.
        #[arg(long, value_name = "TEXT")]
        notes: Option<String>,
        /// The command and its arguments.
        #[arg(
            required = true,
            trailing_var_arg = true,
            allow_hyphen_values = true,
            num_args = 1..
        )]
        cmd: Vec<String>,
    },
    /// Delete a saved command by name (use `project/name` if the name is ambiguous).
    Delete {
        /// Short name, or `project/name`.
        name: String,
    },
    /// Run a saved command by name or `ProjectName/commandName` if names collide.
    Run {
        /// Short name (unique), or `project/command` using the project display name.
        name: String,
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
        Commands::Project { command } => match command {
            ProjectCommands::List => cmd_project_list(&vault_path),
            ProjectCommands::Add { name } => cmd_project_add(&vault_path, &name),
            ProjectCommands::Delete { name } => cmd_project_delete(&vault_path, &name),
        },
        Commands::Saved { command } => match command {
            SavedCommands::List { project } => cmd_saved_list(&vault_path, project.as_deref()),
            SavedCommands::Add { name, project, profile, notes, cmd } => {
                cmd_saved_add(&vault_path, &name, project.as_deref(), profile.as_deref(), notes.as_deref(), cmd)
            }
            SavedCommands::Delete { name } => cmd_saved_delete(&vault_path, &name),
            SavedCommands::Run { name } => cmd_saved_run(&vault_path, &name),
        },
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
    let status = spawn_with_profile_env(&v, Some(profile), cmd)?;
    std::process::exit(status.code().unwrap_or(1));
}

// ── project commands ─────────────────────────────────────────────────────────

fn cmd_project_list(path: &PathBuf) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let v = open_unlocked(path, pw.as_bytes())?;
    let projects = v.list_cli_projects().map_err(fmt_keycard_err)?;
    if projects.is_empty() {
        eprintln!("No projects.");
    } else {
        for p in &projects {
            println!("{}\t{}", p.id, p.name);
        }
    }
    Ok(())
}

fn cmd_project_add(path: &PathBuf, name: &str) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let mut v = open_unlocked(path, pw.as_bytes())?;
    let id = Uuid::new_v4().to_string();
    v.add_cli_project(&id, name).map_err(fmt_keycard_err)?;
    eprintln!("Project '{}' created (id: {}).", name, id);
    Ok(())
}

fn cmd_project_delete(path: &PathBuf, name: &str) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let mut v = open_unlocked(path, pw.as_bytes())?;
    // accept either id or name: resolve by listing
    let projects = v.list_cli_projects().map_err(fmt_keycard_err)?;
    let project_id = projects
        .iter()
        .find(|p| p.id == name || p.name == name)
        .map(|p| p.id.clone())
        .ok_or_else(|| format!("project '{}' not found", name))?;
    v.delete_cli_project(&project_id).map_err(fmt_keycard_err)?;
    eprintln!("Project '{}' deleted.", name);
    Ok(())
}

// ── saved commands ────────────────────────────────────────────────────────────

fn cmd_saved_list(path: &PathBuf, project_filter: Option<&str>) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let v = open_unlocked(path, pw.as_bytes())?;
    let favorites = match project_filter {
        Some(proj) => v.list_cli_favorites_by_project(proj).map_err(fmt_keycard_err)?,
        None => v.list_cli_favorites().map_err(fmt_keycard_err)?,
    };
    for fav in &favorites {
        let argv_json = serde_json::to_string(&fav.argv).map_err(|e| e.to_string())?;
        let prof = fav.profile_id.as_deref().unwrap_or("-");
        println!("{}\t{}\t{}\t{}", fav.project_name, fav.name, prof, argv_json);
    }
    Ok(())
}

fn cmd_saved_add(
    path: &PathBuf,
    name: &str,
    project: Option<&str>,
    profile: Option<&str>,
    notes: Option<&str>,
    cmd: Vec<String>,
) -> Result<(), String> {
    if cmd.is_empty() {
        return Err("command arguments are required".into());
    }
    let pw = read_password("Master password: ")?;
    let mut v = open_unlocked(path, pw.as_bytes())?;

    // Resolve project: by name/id, or default to "General"
    let project_id = if let Some(proj) = project {
        let projects = v.list_cli_projects().map_err(fmt_keycard_err)?;
        projects
            .iter()
            .find(|p| p.id == proj || p.name == proj)
            .map(|p| p.id.clone())
            .ok_or_else(|| format!("project '{}' not found", proj))?
    } else {
        // Use the default "General" project id from core
        keycard_core::DEFAULT_CLI_PROJECT_ID.to_string()
    };

    let id = Uuid::new_v4().to_string();
    v.add_cli_favorite(&id, &project_id, name, profile, &cmd, notes)
        .map_err(fmt_keycard_err)?;
    eprintln!("Saved command '{}' added (id: {}).", name, id);
    Ok(())
}

fn cmd_saved_delete(path: &PathBuf, spec: &str) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let mut v = open_unlocked(path, pw.as_bytes())?;
    // Resolve the favorite id by spec (same project/name logic as run)
    let fav_id = resolve_favorite_id(&v, spec)?;
    v.delete_cli_favorite(&fav_id).map_err(fmt_keycard_err)?;
    eprintln!("Saved command '{}' deleted.", spec);
    Ok(())
}

fn cmd_saved_run(path: &PathBuf, name: &str) -> Result<(), String> {
    let pw = read_password("Master password: ")?;
    let v = open_unlocked(path, pw.as_bytes())?;
    let (argv, profile_id) = v
        .get_cli_favorite_for_run(name)
        .map_err(fmt_keycard_err)?;
    let cmd: Vec<OsString> = argv.into_iter().map(OsString::from).collect();
    let status = spawn_with_profile_env(&v, profile_id.as_deref(), cmd)?;
    std::process::exit(status.code().unwrap_or(1));
}

/// Resolve a saved command's id from a name spec (`name` or `project/name`).
fn resolve_favorite_id(v: &UnlockedVault, spec: &str) -> Result<String, String> {
    let spec = spec.trim();
    if spec.is_empty() {
        return Err("command name is empty".into());
    }
    if let Some((proj_part, cmd_part)) = spec.split_once('/') {
        let proj = proj_part.trim();
        let cmd = cmd_part.trim();
        let favs = v
            .list_cli_favorites_by_project(proj)
            .map_err(fmt_keycard_err)?;
        favs.into_iter()
            .find(|f| f.name == cmd)
            .map(|f| f.id)
            .ok_or_else(|| format!("saved command '{spec}' not found"))
    } else {
        let favs = v.list_cli_favorites().map_err(fmt_keycard_err)?;
        let matches: Vec<_> = favs.iter().filter(|f| f.name == spec).collect();
        match matches.len() {
            0 => Err(format!("saved command '{spec}' not found")),
            1 => Ok(matches[0].id.clone()),
            _ => Err(format!(
                "ambiguous name '{spec}'; use `project/name` to disambiguate"
            )),
        }
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Merge optional Keycard profile env into current env and run `cmd` (non-empty argv).
fn spawn_with_profile_env(
    v: &UnlockedVault,
    profile_name_or_id: Option<&str>,
    cmd: Vec<OsString>,
) -> Result<std::process::ExitStatus, String> {
    let map: BTreeMap<String, String> = match profile_name_or_id {
        Some(p) => env_map(v, p)?,
        None => BTreeMap::new(),
    };
    let mut it = cmd.into_iter();
    let program = it
        .next()
        .ok_or_else(|| "missing program (empty argv)".to_string())?;
    let mut child = std::process::Command::new(program);
    child.args(it);
    child.envs(std::env::vars());
    for (k, val) in map {
        child.env(k, val);
    }
    child.status().map_err(|e| e.to_string())
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
