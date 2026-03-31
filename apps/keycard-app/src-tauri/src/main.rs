#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use keycard_core::{
    init_vault, is_vault_initialized, pending_cli_snippet_path, vault_db_path, CliFavoriteMeta,
    EntryMeta, ProfileMeta, UnlockedVault, Vault,
};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Emitter, Manager};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{Builder as ShortcutBuilder, ShortcutState};

#[derive(Clone)]
struct AppState {
    vault_path: PathBuf,
    dek: Arc<Mutex<Option<[u8; 32]>>>,
}

fn with_unlocked<T>(
    state: &AppState,
    f: impl FnOnce(UnlockedVault) -> Result<T, String>,
) -> Result<T, String> {
    let dek = *state
        .dek
        .lock()
        .map_err(|_| "session lock poisoned".to_string())?
        .as_ref()
        .ok_or_else(|| "vault is locked".to_string())?;
    let v = UnlockedVault::reopen(&state.vault_path, &dek).map_err(|e| e.to_string())?;
    f(v)
}

#[tauri::command]
fn default_vault_path_cmd() -> Result<String, String> {
    vault_db_path()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn is_vault_initialized_cmd(state: tauri::State<AppState>) -> Result<bool, String> {
    is_vault_initialized(&state.vault_path).map_err(|e| e.to_string())
}

#[tauri::command]
fn gui_init_vault_cmd(
    state: tauri::State<AppState>,
    password: String,
    password_confirm: String,
) -> Result<(), String> {
    if password != password_confirm {
        return Err("passwords do not match".into());
    }
    init_vault(&state.vault_path, password.as_bytes()).map_err(|e| e.to_string())
}

#[tauri::command]
fn unlock_vault_cmd(state: tauri::State<AppState>, password: String) -> Result<(), String> {
    let v = Vault::open(&state.vault_path).map_err(|e| e.to_string())?;
    let u = v.unlock(password.as_bytes()).map_err(|e| e.to_string())?;
    *state.dek.lock().map_err(|_| "lock poisoned")? = Some(*u.dek_for_crypto());
    Ok(())
}

#[tauri::command]
fn lock_vault_cmd(state: tauri::State<AppState>) -> Result<(), String> {
    *state.dek.lock().map_err(|_| "lock poisoned")? = None;
    Ok(())
}

#[tauri::command]
fn is_unlocked_cmd(state: tauri::State<AppState>) -> bool {
    state.dek.lock().map(|g| g.is_some()).unwrap_or(false)
}

#[tauri::command]
fn list_entries_json_cmd(state: tauri::State<AppState>) -> Result<Vec<EntryMeta>, String> {
    with_unlocked(&state, |v| v.list_entries_meta().map_err(|e| e.to_string()))
}

#[tauri::command]
fn add_entry_cmd(
    state: tauri::State<AppState>,
    id: String,
    provider: Option<String>,
    alias: String,
    tags: Option<String>,
    secret: String,
    kind: Option<String>,
) -> Result<(), String> {
    let entry_kind = match kind.as_deref() {
        Some("password") => keycard_core::EntryKind::Password,
        _ => keycard_core::EntryKind::Api,
    };
    with_unlocked(&state, |mut v| {
        v.add_entry(
            &id,
            provider.as_deref(),
            &alias,
            tags.as_deref(),
            secret.as_bytes(),
            entry_kind,
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn get_secret_utf8_cmd(state: tauri::State<AppState>, id: String) -> Result<String, String> {
    with_unlocked(&state, |v| {
        let b = v.get_entry_secret(&id).map_err(|e| e.to_string())?;
        String::from_utf8(b).map_err(|_| "secret is not valid UTF-8".to_string())
    })
}

#[tauri::command]
fn get_setting_cmd(state: tauri::State<AppState>, key: String) -> Result<Option<String>, String> {
    with_unlocked(&state, |v| v.get_app_setting(&key).map_err(|e| e.to_string()))
}

#[tauri::command]
fn set_setting_cmd(state: tauri::State<AppState>, key: String, value: String) -> Result<(), String> {
    with_unlocked(&state, |mut v| v.set_app_setting(&key, &value).map_err(|e| e.to_string()))
}

#[tauri::command]
fn read_clipboard_cmd(app: tauri::AppHandle) -> Result<String, String> {
    app.clipboard().read_text().map_err(|e| e.to_string())
}

#[tauri::command]
fn write_clipboard_cmd(app: tauri::AppHandle, text: String) -> Result<(), String> {
    app.clipboard()
        .write_text(&text)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_profiles_json_cmd(state: tauri::State<AppState>) -> Result<Vec<ProfileMeta>, String> {
    with_unlocked(&state, |v| v.list_profiles().map_err(|e| e.to_string()))
}

#[tauri::command]
fn list_cli_favorites_json_cmd(
    state: tauri::State<AppState>,
) -> Result<Vec<CliFavoriteMeta>, String> {
    with_unlocked(&state, |v| v.list_cli_favorites().map_err(|e| e.to_string()))
}

#[tauri::command]
fn add_cli_favorite_cmd(
    state: tauri::State<AppState>,
    id: String,
    name: String,
    profile_id: Option<String>,
    argv: Vec<String>,
    notes: Option<String>,
) -> Result<(), String> {
    with_unlocked(&state, |mut v| {
        let resolved = match profile_id
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            None => None,
            Some(s) => Some(v.resolve_profile_id(s).map_err(|e| e.to_string())?),
        };
        v.add_cli_favorite(
            &id,
            &name,
            resolved.as_deref(),
            &argv,
            notes.as_deref(),
        )
        .map_err(|e| e.to_string())
    })
}

#[tauri::command]
fn delete_cli_favorite_cmd(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    with_unlocked(&state, |mut v| v.delete_cli_favorite(&id).map_err(|e| e.to_string()))
}

/// Read and delete `pending_cli_snippet.txt` (written by macOS Quick Action / helper script).
fn drain_pending_cli_snippet_from_disk() -> Result<Option<String>, String> {
    let path = pending_cli_snippet_path().map_err(|e| e.to_string())?;
    if !path.exists() {
        return Ok(None);
    }
    let s = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&path);
    let t = s.trim();
    if t.is_empty() {
        Ok(None)
    } else {
        Ok(Some(t.to_string()))
    }
}

#[tauri::command]
fn take_pending_cli_snippet_cmd() -> Result<Option<String>, String> {
    drain_pending_cli_snippet_from_disk()
}

/// When any window focuses while unlocked, pull the staging file and send it to the **main** webview.
/// Avoids losing snippets when a `?quick=1` window (no CLI form) was frontmost — JS never called `take_pending` before.
fn try_forward_pending_cli_to_main(app: &tauri::AppHandle, state: &AppState) {
    let unlocked = state
        .dek
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false);
    if !unlocked {
        return;
    }
    let Some(text) = (match drain_pending_cli_snippet_from_disk() {
        Ok(x) => x,
        Err(_) => return,
    }) else {
        return;
    };
    let Some(main) = app.get_webview_window("main") else {
        return;
    };
    let _ = main.emit("pending-cli-snippet", text);
    let _ = main.set_focus();
}

fn main() {
    let vault_path = vault_db_path().unwrap_or_else(|_| PathBuf::from("vault.db"));
    let state = AppState {
        vault_path,
        dek: Arc::new(Mutex::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(
            ShortcutBuilder::new()
                .with_shortcuts(["commandorcontrol+shift+k"])
                .expect("shortcut")
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let _ = app.emit("open-quick-save", ());
                    }
                })
                .build(),
        )
        .manage(state.clone())
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::Focused(true) = event {
                let app = window.app_handle().clone();
                if let Some(s) = app.try_state::<AppState>() {
                    try_forward_pending_cli_to_main(&app, &s);
                }
            }
        })
        .setup(move |app| {
            let show = MenuItem::with_id(app, "show", "Open Keycard", true, None::<&str>)?;
            let quick = MenuItem::with_id(app, "quick", "Save clipboard…", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show, &quick, &quit])?;

            let mut tray = TrayIconBuilder::with_id("keycard-tray")
                .menu(&menu)
                .tooltip("Keycard");
            if let Some(icon) = app.default_window_icon().cloned() {
                tray = tray.icon(icon);
            }
            let _tray = tray
                .on_menu_event(move |app, event| {
                    if event.id == "quit" {
                        app.exit(0);
                    } else if event.id == "show" {
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    } else if event.id == "quick" {
                        let _ = app.emit("open-quick-save", ());
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: tauri::tray::MouseButton::Left,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            let _ = w.show();
                            let _ = w.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            default_vault_path_cmd,
            is_vault_initialized_cmd,
            gui_init_vault_cmd,
            unlock_vault_cmd,
            lock_vault_cmd,
            is_unlocked_cmd,
            list_entries_json_cmd,
            add_entry_cmd,
            get_secret_utf8_cmd,
            get_setting_cmd,
            set_setting_cmd,
            read_clipboard_cmd,
            write_clipboard_cmd,
            list_profiles_json_cmd,
            list_cli_favorites_json_cmd,
            add_cli_favorite_cmd,
            delete_cli_favorite_cmd,
            take_pending_cli_snippet_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
