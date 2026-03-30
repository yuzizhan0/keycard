# Keycard

Local-first, open-source tool to store API keys in an encrypted SQLite vault, with a desktop app (Tauri) and a `keycard` CLI for terminal workflows.

**License:** [MIT](LICENSE) · **Conduct:** [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) · **Open source (zh):** [docs/OPEN_SOURCE.md](docs/OPEN_SOURCE.md) · **Security:** [SECURITY.md](SECURITY.md)

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

From the repo root (workspace):

```bash
npm install
npm run tauri dev    # dev
npm run tauri build  # release .app / .dmg (frontend is built automatically)
```

Or only the app package:

```bash
cd apps/keycard-app
npm install
npm run tauri dev
npm run tauri build
```

**本机当「正规 App」用（macOS，小白向）：**

1. 装好 [Rust](https://rustup.rs/)，并具备 Xcode 命令行工具（首次打包终端里按提示安装）。
2. 在仓库根目录执行：`npm install`，再执行：`npm run tauri build`（会跑几分钟）。
3. 产物在 `apps/keycard-app/src-tauri/target/release/bundle/macos/`：里面有 **`Keycard.app`**，以及 **`Keycard_*.dmg`**（若生成了 DMG）。
4. **最省事**：打开 `.dmg`，把 **Keycard** 拖到 **应用程序** 文件夹；之后用 Launchpad 或聚焦搜索「Keycard」打开。
5. 未做 Apple 公证的构建，**第一次**打开请在 **应用程序** 里对 Keycard **右键 → 打开**，在提示里选「打开」；以后可正常从 Dock / 启动台点开。

**Port 1420 already in use:** Vite is pinned to `1420` (see `vite.config.ts` / `tauri.conf.json`). Quit the other dev session, or free the port on macOS/Linux:

```bash
cd apps/keycard-app && npm run free-port
# or: lsof -ti:1420 | xargs kill -9
```

Then run `npm run tauri dev` again.

**Tauri commands (frontend):** argument names from Rust `snake_case` map to **camelCase** in `invoke()`, e.g. `password_confirm` → `passwordConfirm`.

## CLI

**首次配置（强烈推荐按顺序读）：**

- **macOS：** [docs/cli-setup-macos.md](docs/cli-setup-macos.md)（PATH、默认 `vault.db`、`env`/`run`、Profile、与桌面共用保险库）
- **Windows：** [docs/cli-setup-windows.md](docs/cli-setup-windows.md)（MSVC、`keycard.exe`、`%LOCALAPPDATA%`、`env` 与 PowerShell/cmd 的差异、优先 `run`）

```bash
# Create vault (interactive password)
keycard init

# POSIX exports for a profile (after you add entries + profile mappings in the app or future CLI)
keycard env --profile dev
# eval "$(keycard env --profile dev)"

# Run a command with injected env
keycard run --profile dev -- cargo build

# Saved commands (managed in the desktop app under “Saved CLI commands”)
keycard saved list
keycard saved run my-build
```

Override vault path: `keycard --vault /path/to/vault.db …`.

Saved commands store `argv` as a JSON array in the vault. `saved run` merges the optional profile’s env vars (same as `run -p`) and spawns the program. Profile rows still live in `profiles` / `profile_env` (create them via SQL or a future UI).

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
- Main window: **Saved CLI commands** — name, optional profile, program + space-split arguments; copy the saved command line to the clipboard; `keycard saved list` / `keycard saved run` in the CLI.

### macOS: save a terminal selection into “CLI commands”

Terminal apps do not allow third-party items in their context menu. Use a **Quick Action** (shows under **Services** / some apps under the right-click menu):

1. `chmod +x scripts/macos/save-cli-to-keycard.sh` (once).
2. Open **Automator** → **Quick Action** → “Workflow receives current **text**” in **any application**.
3. Add action **Run Shell Script**. Set **传递输入** to **标准输入** (stdin)—not “作为自变量”—then script:
   ```bash
   /bin/bash /absolute/path/to/keycard/scripts/macos/save-cli-to-keycard.sh
   ```
   The script also falls back to shell arguments if stdin is empty (wrong Automator setting), but quoted commands may break; prefer stdin.
4. Save as **Save CLI to Keycard** (or any name you prefer). In **System Settings → Keyboard → Keyboard Shortcuts → Services**, enable it if needed.
5. In **Terminal.app**, select text → right-click → **Services** → **Save CLI to Keycard**. **Keycard must be running**; the vault must be **unlocked** (if it was locked, unlock after running the service—the snippet is still picked up). The main window switches to the CLI tab and prefills **Program** / **Arguments** from the first non-empty line (split on spaces).

The helper writes `~/Library/Application Support/Keycard/pending_cli_snippet.txt`; the app reads and deletes it. The **main** window also receives a native focus hook so the file is drained even when a **`?quick=1`** window was frontmost (the old JS-only path never called `take_pending` without `#cli-add-form` on that webview). For **`tauri dev`**, AppleScript activation may not find the app by bundle id — click the Keycard window after the service if needed; a built **`.app`** works more reliably.

Shell support for `keycard env`: POSIX `export` only in v1 (fish/pwsh not targeted).

## Trademark

“Keycard” and the Keycard logo are trademarks of ZizhanYu.
You may use this project’s source code under the LICENSE, but you may not use
the marks to imply endorsement or an official distribution without permission.