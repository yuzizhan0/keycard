# Keycard

Local-first, open-source tool to store API keys in an encrypted SQLite vault, with a desktop app (Tauri) and a `keycard` CLI for terminal workflows.

## Data location

Default `vault.db` path (see design spec §2):

| OS      | Path |
|--------|------|
| macOS  | `~/Library/Application Support/Keycard/vault.db` |
| Linux  | `$XDG_DATA_HOME/Keycard/vault.db` or `~/.local/share/Keycard/vault.db` |
| Windows | `%LOCALAPPDATA%\Keycard\vault.db` |

## Build

**Rust:** stable toolchain with Cargo.

```bash
cargo build -p keycard-core
cargo build -p keycard-cli
# binary: target/debug/keycard
```

**Desktop app:**

```bash
cd apps/keycard-app
npm install
npm run tauri dev    # dev
npm run tauri build  # release (requires frontend build)
```

## CLI

```bash
# Create vault (interactive password)
keycard init

# POSIX exports for a profile (after you add entries + profile mappings in the app or future CLI)
keycard env --profile dev
# eval "$(keycard env --profile dev)"

# Run a command with injected env
keycard run --profile dev -- cargo build
```

Override vault path: `keycard --vault /path/to/vault.db …`.

### Tests / CI password (not for production)

If `KEYCARD_ALLOW_ENV_PASSWORD=1`, the CLI reads the master password from `KEYCARD_MASTER_PASSWORD` instead of prompting. **Do not set this in production shells.**

## Security notes

- Master password is verified with an encrypted sentinel stored in `meta` (`pw_verify_*`); wrong passwords fail unlock.
- Error messages and logs must not contain decrypted secrets; see `docs/MANUAL_TEST.md` for spot-checks.
- Threat model: local malware, backup leaks, shared machines — see `docs/superpowers/specs/2026-03-29-keycard-design.md`.

## Design & plan

- Spec: `docs/superpowers/specs/2026-03-29-keycard-design.md`
- Implementation plan: `docs/superpowers/plans/2026-03-29-keycard.md`

## App features (v1)

- Tray menu: Open, Save clipboard…, Quit; shortcut **⌘⇧K** / **Ctrl+Shift+K** (global) opens quick-save when unlocked.
- Quick-save window: clipboard prefilled; optional shape hints (`sk-`, `Bearer`); optional clear clipboard after save (setting).
- Main window: unlock, list/search entries, add entry, copy secret (optional clear clipboard after N seconds), idle lock (setting), persisted in `app_settings`.

Shell support for `keycard env`: POSIX `export` only in v1 (fish/pwsh not targeted).
