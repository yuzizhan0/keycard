import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import {
  langSelectHtml,
  providerPresetSelectHtml,
  setLocale,
  t,
  tf,
  type Locale,
  windowTitleQuickSave,
} from "./i18n";
import { PROVIDER_PRESET_VALUE_SET } from "./provider-presets";
import {
  getTheme,
  initTheme,
  themeToggleInnerHtml,
  toggleTheme,
} from "./theme";

initTheme();

type EntryMeta = {
  id: string;
  provider: string | null;
  alias: string;
  tags: string | null;
  created_at: number;
  kind: "api" | "password";
};

type ProfileMeta = {
  id: string;
  name: string;
};

type CliFavoriteMeta = {
  id: string;
  name: string;
  profile_id: string | null;
  argv: string[];
  notes: string | null;
};

async function copyEntrySecretToClipboard(id: string): Promise<void> {
  try {
    const text = await invoke<string>("get_secret_utf8_cmd", { id });
    await invoke("write_clipboard_cmd", { text });
    const sec = await invoke<string | null>("get_setting_cmd", {
      key: "clear_clipboard_after_copy_sec",
    });
    const n = sec ? parseInt(sec, 10) : 0;
    if (n > 0) {
      setTimeout(() => {
        invoke("write_clipboard_cmd", { text: "" }).catch(() => {});
      }, n * 1000);
    }
  } catch (x) {
    alert(String(x));
  }
}

const appEl = document.querySelector<HTMLDivElement>("#app")!;

/** Re-render after language change (set by each full-screen view). */
let rerenderCurrent: (() => Promise<void>) | null = null;

let settingsModalEscHandler: ((e: KeyboardEvent) => void) | null = null;

function clearSettingsModalEsc() {
  if (settingsModalEscHandler) {
    document.removeEventListener("keydown", settingsModalEscHandler);
    settingsModalEscHandler = null;
  }
}

function closeSettingsModal() {
  clearSettingsModalEsc();
  const m = document.getElementById("settings-modal");
  if (m) {
    m.classList.remove("is-open");
    m.setAttribute("aria-hidden", "true");
  }
}

function openSettingsModal() {
  const m = document.getElementById("settings-modal");
  if (!m) return;
  clearSettingsModalEsc();
  settingsModalEscHandler = (e: KeyboardEvent) => {
    if (e.key === "Escape") closeSettingsModal();
  };
  document.addEventListener("keydown", settingsModalEscHandler);
  m.classList.add("is-open");
  m.setAttribute("aria-hidden", "false");
  document.getElementById("settings-close")?.focus();
}

function settingsModalHtml(settings: Record<string, string>): string {
  return `<div id="settings-modal" class="modal" aria-hidden="true">
  <button type="button" class="modal-backdrop" id="settings-modal-backdrop" aria-label="${escapeHtml(t("settingsCloseAria"))}"></button>
  <div class="modal-panel panel" role="dialog" aria-modal="true" aria-labelledby="settings-modal-title">
    <div class="modal-header">
      <h2 id="settings-modal-title" class="modal-title">${escapeHtml(t("mainSettings"))}</h2>
      <button type="button" id="settings-close" class="modal-close secondary" aria-label="${escapeHtml(t("settingsCloseAria"))}">×</button>
    </div>
    <form id="set-form" class="stack">
      <label>${escapeHtml(t("mainIdleLock"))}</label>
      <input name="idle_lock_minutes" type="number" min="0" value="${settings.idle_lock_minutes ?? "15"}" />
      <label>${escapeHtml(t("mainClearClipQuickSave"))}</label>
      <select name="clear_clipboard_on_save">
        <option value="0" ${settings.clear_clipboard_on_save === "1" ? "" : "selected"}>${escapeHtml(t("mainNo"))}</option>
        <option value="1" ${settings.clear_clipboard_on_save === "1" ? "selected" : ""}>${escapeHtml(t("mainYes"))}</option>
      </select>
      <label>${escapeHtml(t("mainClearClipAfterCopy"))}</label>
      <input name="clear_clipboard_after_copy_sec" type="number" min="0" value="${settings.clear_clipboard_after_copy_sec ?? "0"}" />
      <button type="submit">${escapeHtml(t("mainSaveSettings"))}</button>
      <p class="err" id="set-err"></p>
    </form>
  </div>
</div>`;
}

function wireLangSelect() {
  const sel = document.querySelector<HTMLSelectElement>("#lang-select");
  if (!sel) return;
  sel.addEventListener("change", async () => {
    setLocale(sel.value as Locale);
    if (rerenderCurrent) await rerenderCurrent();
  });
}

function wireThemeToggle() {
  const btn = document.getElementById("theme-toggle");
  if (!btn) return;
  btn.addEventListener("click", () => {
    toggleTheme();
    const dark = getTheme() === "dark";
    btn.setAttribute(
      "aria-label",
      dark ? t("themeSwitchToLight") : t("themeSwitchToDark"),
    );
    const wrap = btn.querySelector(".theme-icon");
    if (wrap) wrap.innerHTML = themeToggleInnerHtml();
  });
}

function wireHeader() {
  wireLangSelect();
  wireThemeToggle();
}

function wireProviderPreset(formSelector: string) {
  const form = document.querySelector<HTMLElement>(formSelector);
  if (!form) return;
  const sel = form.querySelector<HTMLSelectElement>("#provider-preset");
  const input = form.querySelector<HTMLInputElement>('input[name="provider"]');
  if (!sel || !input) return;
  sel.addEventListener("change", () => {
    if (sel.value) input.value = sel.value;
  });
  input.addEventListener("input", () => {
    const v = input.value.trim();
    if (PROVIDER_PRESET_VALUE_SET.has(v)) sel.value = v;
    else sel.value = "";
  });
}

function shapeHint(secret: string): string | null {
  const s = secret.trim();
  if (s.startsWith("sk-")) return t("shapeOpenai");
  if (/^Bearer\s+/i.test(s)) return t("shapeBearer");
  return null;
}

function render(html: string) {
  clearSettingsModalEsc();
  appEl.innerHTML = html;
}

async function defaultPath(): Promise<string> {
  return invoke<string>("default_vault_path_cmd");
}

/** `true` when `vault.db` exists and contains vault metadata (user should unlock, not create). */
async function isVaultInitialized(): Promise<boolean> {
  return invoke<boolean>("is_vault_initialized_cmd");
}

function wireUnlock() {
  const form = document.querySelector<HTMLFormElement>("#unlock-form")!;
  const err = document.querySelector<HTMLParagraphElement>("#unlock-err")!;
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    err.textContent = "";
    const fd = new FormData(form);
    const password = String(fd.get("password") || "");
    try {
      await invoke("unlock_vault_cmd", { password });
      await showMain();
    } catch (x) {
      err.textContent = String(x);
    }
  });
}

function wireInit() {
  const form = document.querySelector<HTMLFormElement>("#init-form")!;
  const err = document.querySelector<HTMLParagraphElement>("#init-err")!;
  form.addEventListener("submit", async (e) => {
    e.preventDefault();
    err.textContent = "";
    const fd = new FormData(form);
    const a = String(fd.get("p1") || "");
    const b = String(fd.get("p2") || "");
    try {
      await invoke("gui_init_vault_cmd", {
        password: a,
        passwordConfirm: b,
      });
      await showUnlock();
    } catch (x) {
      err.textContent = String(x);
    }
  });
}

async function loadSettings(): Promise<Record<string, string>> {
  const keys = [
    "idle_lock_minutes",
    "clear_clipboard_on_save",
    "clear_clipboard_after_copy_sec",
  ];
  const out: Record<string, string> = {};
  for (const k of keys) {
    const v = await invoke<string | null>("get_setting_cmd", { key: k });
    if (v != null) out[k] = v;
  }
  return out;
}

let lastActivity = Date.now();
function bumpActivity() {
  lastActivity = Date.now();
}
document.addEventListener("click", bumpActivity);
document.addEventListener("keydown", bumpActivity);

let idleIntervalId: ReturnType<typeof setInterval> | null = null;

function clearIdleLoop() {
  if (idleIntervalId !== null) {
    clearInterval(idleIntervalId);
    idleIntervalId = null;
  }
}

/** Single interval; only runs `get_setting_cmd` while vault reports unlocked (avoids IPC errors after Lock). */
function startIdleLoop() {
  clearIdleLoop();
  idleIntervalId = setInterval(async () => {
    try {
      const unlocked = await invoke<boolean>("is_unlocked_cmd");
      if (!unlocked) return;
      const mins = await invoke<string | null>("get_setting_cmd", {
        key: "idle_lock_minutes",
      });
      const m = mins ? parseInt(mins, 10) : 0;
      if (!m || m <= 0) return;
      if (Date.now() - lastActivity > m * 60_000) {
        await invoke("lock_vault_cmd");
        clearIdleLoop();
        await showUnlock();
      }
    } catch {
      /* locked between checks or transient IPC */
    }
  }, 15_000);
}

function profileOptionsHtml(profiles: ProfileMeta[]): string {
  const parts = [`<option value="">${escapeAttr(t("cliProfileNone"))}</option>`];
  for (const p of profiles) {
    parts.push(
      `<option value="${escapeAttr(p.id)}">${escapeHtml(p.name)}</option>`,
    );
  }
  return parts.join("");
}

/** Split one shell-like string into argv tokens; respects `"` and `'` (no escapes inside quotes). */
function splitShellLikeTokens(s: string): string[] {
  const out: string[] = [];
  let cur = "";
  let quote: '"' | "'" | null = null;
  for (let i = 0; i < s.length; i++) {
    const c = s[i]!;
    if (quote) {
      if (c === quote) quote = null;
      else cur += c;
    } else if (c === '"' || c === "'") {
      quote = c;
    } else if (/\s/.test(c)) {
      if (cur.length > 0) {
        out.push(cur);
        cur = "";
      }
    } else {
      cur += c;
    }
  }
  if (cur.length > 0) out.push(cur);
  return out;
}

function shellQuoteArgIfNeeded(a: string): string {
  if (a === "") return '""';
  if (/[\s'"\\]/.test(a)) {
    return `"${a.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
  }
  return a;
}

/** Join argv into a pasteable shell line (matches “Copy command”). */
function formatArgvForShellCopy(argv: string[]): string {
  return argv.map(shellQuoteArgIfNeeded).join(" ");
}

function formatArgsForForm(tail: string[]): string {
  return tail.map(shellQuoteArgIfNeeded).join(" ");
}

function stripLeadingShellPrompt(line: string): string {
  let s = line.trimStart();
  if (s.length >= 2 && (s[0] === "$" || s[0] === "#") && /\s/.test(s[1]!)) {
    s = s.slice(1).trimStart();
  }
  return s;
}

function firstNonEmptyShellLine(text: string): string {
  const line =
    text
      .trim()
      .split(/\r?\n/)
      .map((l) => l.trim())
      .find((l) => l.length > 0) ?? "";
  return stripLeadingShellPrompt(line);
}

function argvFromForm(program: string, argsLine: string): string[] {
  const p = program.trim();
  if (!p) return [""];
  return [p, ...splitShellLikeTokens(argsLine.trim())];
}

const MAIN_TAB_STORAGE = "keycard_main_tab";
type MainVaultTab = "entries" | "passwords" | "cli";

function getMainVaultTab(): MainVaultTab {
  try {
    const v = localStorage.getItem(MAIN_TAB_STORAGE);
    if (v === "cli") return "cli";
    if (v === "passwords") return "passwords";
  } catch {
    /* private mode */
  }
  return "entries";
}

function setMainVaultTab(tab: MainVaultTab) {
  try {
    localStorage.setItem(MAIN_TAB_STORAGE, tab);
  } catch {
    /* ignore */
  }
}

let pendingCliFocusUnlisten: (() => void) | null = null;

/** Payload from Rust `pending-cli-snippet` when DOM was not ready (e.g. unlock screen). */
let queuedCliSnippetPayload: string | null = null;

/** First non-empty line → program + args string (same rules as manual Arguments field). */
function parseTerminalSelectionToProgramArgs(text: string): {
  program: string;
  args: string;
} {
  const line = firstNonEmptyShellLine(text);
  const tokens = splitShellLikeTokens(line);
  if (tokens.length === 0) return { program: "", args: "" };
  const program = tokens[0] ?? "";
  const args = formatArgsForForm(tokens.slice(1));
  return { program, args };
}

/** After Services / AppleScript activates the window, focus can lag; poll a few times. */
function schedulePendingCliSnippetPulls(): void {
  const delaysMs = [0, 200, 600];
  for (const ms of delaysMs) {
    window.setTimeout(() => {
      void fillPendingCliSnippetFromDisk();
    }, ms);
  }
}

async function ensurePendingCliSnippetFocusListener(): Promise<void> {
  if (pendingCliFocusUnlisten) return;
  try {
    pendingCliFocusUnlisten = await getCurrentWindow().onFocusChanged(
      ({ payload: focused }) => {
        if (focused) schedulePendingCliSnippetPulls();
      },
    );
  } catch {
    pendingCliFocusUnlisten = () => {};
  }
  document.addEventListener("visibilitychange", () => {
    if (document.visibilityState === "visible") {
      schedulePendingCliSnippetPulls();
    }
  });
}

function applyCliInputsFromRaw(raw: string): void {
  document.getElementById("tab-main-cli")?.click();
  const { program, args } = parseTerminalSelectionToProgramArgs(raw);
  const apply = () => {
    const progInput = document.querySelector<HTMLInputElement>(
      '#cli-add-form input[name="program"]',
    );
    const argsInput = document.querySelector<HTMLInputElement>(
      '#cli-add-form input[name="args"]',
    );
    if (progInput) progInput.value = program;
    if (argsInput) argsInput.value = args;
    progInput?.focus();
  };
  requestAnimationFrame(() => requestAnimationFrame(apply));
}

/** Apply text from disk (JS path) or from Rust `pending-cli-snippet` after file was already drained. */
async function acceptCliSnippetPayload(raw: string): Promise<void> {
  const unlocked = await invoke<boolean>("is_unlocked_cmd").catch(() => false);
  if (!unlocked || !document.querySelector("#cli-add-form")) {
    queuedCliSnippetPayload = raw;
    return;
  }
  queuedCliSnippetPayload = null;
  applyCliInputsFromRaw(raw);
}

function flushQueuedCliSnippetPayload(): void {
  const raw = queuedCliSnippetPayload;
  if (!raw) return;
  queuedCliSnippetPayload = null;
  void acceptCliSnippetPayload(raw);
}

/** Consume `pending_cli_snippet.txt` and prefill the CLI form (main window, unlocked). */
async function fillPendingCliSnippetFromDisk(): Promise<void> {
  const unlocked = await invoke<boolean>("is_unlocked_cmd").catch(() => false);
  if (!unlocked) return;
  if (!document.querySelector("#cli-add-form")) return;
  const raw = await invoke<string | null>("take_pending_cli_snippet_cmd").catch(
    () => null,
  );
  if (raw == null || raw === "") return;
  applyCliInputsFromRaw(raw);
}

async function showMain() {
  rerenderCurrent = showMain;
  const entries = await invoke<EntryMeta[]>("list_entries_json_cmd");
  const profiles = await invoke<ProfileMeta[]>("list_profiles_json_cmd");
  const cliFavs = await invoke<CliFavoriteMeta[]>("list_cli_favorites_json_cmd");
  const settings = await loadSettings();
  const initialTab = getMainVaultTab();
  const tabEntriesActive = initialTab === "entries";
  const tabPasswordsActive = initialTab === "passwords";
  const tabCliActive = initialTab === "cli";

  render(`
    ${langSelectHtml({ showSettingsGear: true })}
    <div class="page">
      <div class="page-head-main">
        <h1 class="page-title">${escapeHtml(t("mainTitle"))}</h1>
        <button type="button" class="secondary btn-compact" id="lock">${escapeHtml(t("mainLock"))}</button>
      </div>
      <div class="main-tabs" role="tablist" aria-label="${escapeAttr(t("mainTabsAria"))}">
        <button type="button" class="main-tab" role="tab" id="tab-main-entries" aria-selected="${tabEntriesActive}" aria-controls="panel-main-entries" tabindex="${tabEntriesActive ? 0 : -1}">${escapeHtml(t("mainTabSecrets"))}</button>
        <button type="button" class="main-tab" role="tab" id="tab-main-passwords" aria-selected="${tabPasswordsActive}" aria-controls="panel-main-passwords" tabindex="${tabPasswordsActive ? 0 : -1}">${escapeHtml(t("mainTabPasswords"))}</button>
        <button type="button" class="main-tab" role="tab" id="tab-main-cli" aria-selected="${tabCliActive}" aria-controls="panel-main-cli" tabindex="${tabCliActive ? 0 : -1}">${escapeHtml(t("mainTabCli"))}</button>
      </div>
      <div id="panel-main-entries" class="tab-panel" role="tabpanel" aria-labelledby="tab-main-entries" ${tabEntriesActive ? "" : "hidden"}>
        <section class="panel">
          <div class="toolbar">
            <input type="search" id="q" placeholder="${escapeHtml(t("mainSearchPlaceholder"))}" />
          </div>
          <div class="table-wrap">
            <table class="data-table"><thead><tr><th>${escapeHtml(t("mainColAlias"))}</th><th>${escapeHtml(t("mainColProvider"))}</th><th class="col-action">${escapeHtml(t("mainColActions"))}</th></tr></thead><tbody id="rows"></tbody></table>
          </div>
        </section>
        <section class="panel">
          <h2 class="panel-title">${escapeHtml(t("mainAddEntry"))}</h2>
          <form id="add-form" class="stack">
            <label>${escapeHtml(t("labelAlias"))}</label><input name="alias" required autocomplete="off" />
            <label>${escapeHtml(t("labelProviderPreset"))}</label>
            ${providerPresetSelectHtml()}
            <p class="field-hint">${escapeHtml(t("providerPresetHint"))}</p>
            <label>${escapeHtml(t("labelProvider"))}</label><input name="provider" />
            <label>${escapeHtml(t("labelTags"))}</label><input name="tags" />
            <label>${escapeHtml(t("labelSecret"))}</label><textarea name="secret" rows="3" required></textarea>
            <p class="hint" id="shape-hint"></p>
            <button type="submit">${escapeHtml(t("mainSaveEntry"))}</button>
            <p class="err" id="add-err"></p>
          </form>
        </section>
      </div>
      <div id="panel-main-passwords" class="tab-panel" role="tabpanel" aria-labelledby="tab-main-passwords" ${tabPasswordsActive ? "" : "hidden"}>
        <section class="panel">
          <div class="toolbar">
            <input type="search" id="q-password" placeholder="${escapeHtml(t("mainSearchPlaceholder"))}" />
          </div>
          <div class="table-wrap">
            <table class="data-table"><thead><tr><th>${escapeHtml(t("mainColAlias"))}</th><th>${escapeHtml(t("mainColNote"))}</th><th class="col-action">${escapeHtml(t("mainColActions"))}</th></tr></thead><tbody id="password-rows"></tbody></table>
          </div>
        </section>
        <section class="panel">
          <h2 class="panel-title">${escapeHtml(t("mainAddPassword"))}</h2>
          <p class="field-hint">${escapeHtml(t("mainPasswordSectionHint"))}</p>
          <form id="password-add-form" class="stack">
            <label>${escapeHtml(t("labelAlias"))}</label><input name="alias" required autocomplete="off" />
            <label>${escapeHtml(t("labelTags"))}</label><input name="tags" />
            <label>${escapeHtml(t("labelPasswordValue"))}</label><textarea name="secret" rows="3" required autocomplete="off"></textarea>
            <button type="submit">${escapeHtml(t("mainSaveEntry"))}</button>
            <p class="err" id="password-add-err"></p>
          </form>
        </section>
      </div>
      <div id="panel-main-cli" class="tab-panel" role="tabpanel" aria-labelledby="tab-main-cli" ${tabCliActive ? "" : "hidden"}>
        <section class="panel">
          <h2 class="panel-title">${escapeHtml(t("cliSectionTitle"))}</h2>
          <p class="field-hint">${escapeHtml(t("cliSectionHint"))}</p>
          <p class="field-hint">${escapeHtml(t("cliProfileHint"))}</p>
          <p class="field-hint">${escapeHtml(t("cliTerminalHint"))}</p>
          <div class="table-wrap">
            <table class="data-table"><thead><tr><th>${escapeHtml(t("cliColName"))}</th><th>${escapeHtml(t("cliColProfile"))}</th><th>${escapeHtml(t("cliColCommand"))}</th><th class="col-action">${escapeHtml(t("cliColActions"))}</th></tr></thead><tbody id="cli-rows"></tbody></table>
          </div>
          <form id="cli-add-form" class="stack">
            <label>${escapeHtml(t("labelCliName"))}</label><input name="name" required autocomplete="off" />
            <label>${escapeHtml(t("labelCliProfile"))}</label>
            <select name="profile_id">${profileOptionsHtml(profiles)}</select>
            <label>${escapeHtml(t("labelCliProgram"))}</label><input name="program" required autocomplete="off" />
            <label>${escapeHtml(t("labelCliArgs"))}</label><input name="args" autocomplete="off" />
            <p class="field-hint">${escapeHtml(t("labelCliArgsHint"))}</p>
            <label>${escapeHtml(t("labelCliNotes"))}</label><input name="notes" autocomplete="off" />
            <button type="submit">${escapeHtml(t("cliSaveCommand"))}</button>
            <p class="err" id="cli-err"></p>
          </form>
        </section>
      </div>
    </div>
    ${settingsModalHtml(settings)}
  `);

  wireHeader();
  wireProviderPreset("#add-form");

  const tabBtnEntries = document.getElementById("tab-main-entries")!;
  const tabBtnPasswords = document.getElementById("tab-main-passwords")!;
  const tabBtnCli = document.getElementById("tab-main-cli")!;
  const panelEntries = document.getElementById("panel-main-entries")!;
  const panelPasswords = document.getElementById("panel-main-passwords")!;
  const panelCli = document.getElementById("panel-main-cli")!;

  function activateMainTab(tab: MainVaultTab) {
    const isEntries = tab === "entries";
    const isPasswords = tab === "passwords";
    const isCli = tab === "cli";
    tabBtnEntries.setAttribute("aria-selected", String(isEntries));
    tabBtnPasswords.setAttribute("aria-selected", String(isPasswords));
    tabBtnCli.setAttribute("aria-selected", String(isCli));
    tabBtnEntries.tabIndex = isEntries ? 0 : -1;
    tabBtnPasswords.tabIndex = isPasswords ? 0 : -1;
    tabBtnCli.tabIndex = isCli ? 0 : -1;
    panelEntries.toggleAttribute("hidden", !isEntries);
    panelPasswords.toggleAttribute("hidden", !isPasswords);
    panelCli.toggleAttribute("hidden", !isCli);
    setMainVaultTab(tab);
  }

  tabBtnEntries.addEventListener("click", () => {
    activateMainTab("entries");
    tabBtnEntries.focus();
  });
  tabBtnPasswords.addEventListener("click", () => {
    activateMainTab("passwords");
    tabBtnPasswords.focus();
  });
  tabBtnCli.addEventListener("click", () => {
    activateMainTab("cli");
    tabBtnCli.focus();
  });

  document.getElementById("open-settings")?.addEventListener("click", () => {
    openSettingsModal();
  });
  document
    .getElementById("settings-modal-backdrop")
    ?.addEventListener("click", () => closeSettingsModal());
  document
    .getElementById("settings-close")
    ?.addEventListener("click", () => closeSettingsModal());
  startIdleLoop();

  const apiEntries = entries.filter((e) => e.kind === "api");
  const passwordEntries = entries.filter((e) => e.kind === "password");

  const tbody = document.querySelector<HTMLTableSectionElement>("#rows")!;
  const q = document.querySelector<HTMLInputElement>("#q")!;
  const passTbody =
    document.querySelector<HTMLTableSectionElement>("#password-rows")!;
  const qPass = document.querySelector<HTMLInputElement>("#q-password")!;

  function passwordRowHtml(e: EntryMeta) {
    const note = e.tags ?? "";
    const tags = note.toLowerCase();
    return `<tr data-alias="${e.alias.toLowerCase()}" data-tags="${escapeHtml(tags)}">
      <td>${escapeHtml(e.alias)}</td><td>${escapeHtml(note)}</td>
      <td class="col-action"><button type="button" class="secondary copy-password btn-inline" data-id="${escapeHtml(e.id)}">${escapeHtml(t("mainCopy"))}</button></td>
    </tr>`;
  }

  function applyPasswordFilter() {
    const qq = qPass.value.trim().toLowerCase();
    passTbody.querySelectorAll("tr").forEach((tr) => {
      const a = tr.getAttribute("data-alias") || "";
      const tg = tr.getAttribute("data-tags") || "";
      tr.style.display =
        !qq || a.includes(qq) || tg.includes(qq) ? "" : "none";
    });
  }

  function refreshPasswordTable(list: EntryMeta[]) {
    passTbody.innerHTML = list.map(passwordRowHtml).join("");
    passTbody.querySelectorAll(".copy-password").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const id = (btn as HTMLButtonElement).dataset.id!;
        await copyEntrySecretToClipboard(id);
      });
    });
    applyPasswordFilter();
  }

  async function reloadEntryTables() {
    const list = await invoke<EntryMeta[]>("list_entries_json_cmd");
    refreshTable(list.filter((e) => e.kind === "api"));
    refreshPasswordTable(list.filter((e) => e.kind === "password"));
  }

  function rowHtml(e: EntryMeta) {
    const prov = e.provider ?? "";
    const tags = (e.tags ?? "").toLowerCase();
    return `<tr data-alias="${e.alias.toLowerCase()}" data-prov="${prov.toLowerCase()}" data-tags="${escapeHtml(tags)}">
      <td>${escapeHtml(e.alias)}</td><td>${escapeHtml(prov)}</td>
      <td class="col-action"><button type="button" class="secondary copy btn-inline" data-id="${escapeHtml(e.id)}">${escapeHtml(t("mainCopy"))}</button></td>
    </tr>`;
  }

  function applyFilter() {
    const qq = q.value.trim().toLowerCase();
    tbody.querySelectorAll("tr").forEach((tr) => {
      const a = tr.getAttribute("data-alias") || "";
      const p = tr.getAttribute("data-prov") || "";
      const tg = tr.getAttribute("data-tags") || "";
      tr.style.display =
        !qq || a.includes(qq) || p.includes(qq) || tg.includes(qq)
          ? ""
          : "none";
    });
  }

  function refreshTable(list: EntryMeta[]) {
    tbody.innerHTML = list.map(rowHtml).join("");
    tbody.querySelectorAll(".copy").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const id = (btn as HTMLButtonElement).dataset.id!;
        await copyEntrySecretToClipboard(id);
      });
    });
    applyFilter();
  }

  refreshTable(apiEntries);
  refreshPasswordTable(passwordEntries);
  q.addEventListener("input", applyFilter);
  qPass.addEventListener("input", applyPasswordFilter);

  const cliTbody = document.querySelector<HTMLTableSectionElement>("#cli-rows")!;

  function profileLabel(pid: string | null): string {
    if (!pid) return "—";
    const p = profiles.find((x) => x.id === pid);
    return p ? p.name : `${pid.slice(0, 8)}…`;
  }

  function cliRowHtml(f: CliFavoriteMeta) {
    const cmdLine = formatArgvForShellCopy(f.argv);
    const cmdEsc = escapeAttr(cmdLine);
    const cmdVis = escapeHtml(cmdLine);
    const prof = escapeHtml(profileLabel(f.profile_id));
    const argvAttr = escapeAttr(JSON.stringify(f.argv));
    return `<tr>
      <td>${escapeHtml(f.name)}</td><td>${prof}</td><td title="${cmdEsc}">${cmdVis}</td>
      <td class="col-action">
        <button type="button" class="copy-cli-argv btn-inline" data-argv="${argvAttr}" title="${escapeAttr(t("cliCopyArgvTitle"))}">${escapeHtml(t("cliCopyArgv"))}</button>
        <button type="button" class="secondary cli-del btn-inline" data-id="${escapeAttr(f.id)}">${escapeHtml(t("cliDelete"))}</button>
      </td>
    </tr>`;
  }

  function wireCliRows(list: CliFavoriteMeta[]) {
    cliTbody.innerHTML = list.map(cliRowHtml).join("");
    cliTbody.querySelectorAll(".copy-cli-argv").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const raw = (btn as HTMLButtonElement).dataset.argv;
        if (!raw) return;
        try {
          const argv = JSON.parse(raw) as string[];
          await invoke("write_clipboard_cmd", {
            text: formatArgvForShellCopy(argv),
          });
        } catch (x) {
          alert(String(x));
        }
      });
    });
    cliTbody.querySelectorAll(".cli-del").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const id = (btn as HTMLButtonElement).dataset.id;
        if (!id) return;
        try {
          await invoke("delete_cli_favorite_cmd", { id });
          const next = await invoke<CliFavoriteMeta[]>(
            "list_cli_favorites_json_cmd",
          );
          wireCliRows(next);
        } catch (x) {
          alert(String(x));
        }
      });
    });
  }

  wireCliRows(cliFavs);

  document.querySelector("#cli-add-form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const err = document.querySelector<HTMLParagraphElement>("#cli-err")!;
    err.textContent = "";
    const fd = new FormData(e.target as HTMLFormElement);
    const program = String(fd.get("program") || "");
    const argsLine = String(fd.get("args") || "");
    const argv = argvFromForm(program, argsLine);
    if (!argv[0]) {
      err.textContent = t("cliErrProgram");
      return;
    }
    const profRaw = String(fd.get("profile_id") || "").trim();
    try {
      await invoke("add_cli_favorite_cmd", {
        id: crypto.randomUUID(),
        name: String(fd.get("name") || "").trim(),
        profileId: profRaw === "" ? null : profRaw,
        argv,
        notes: String(fd.get("notes") || "").trim() || null,
      });
      (e.target as HTMLFormElement).reset();
      const next = await invoke<CliFavoriteMeta[]>(
        "list_cli_favorites_json_cmd",
      );
      wireCliRows(next);
    } catch (x) {
      err.textContent = String(x);
    }
  });

  document.querySelector("#lock")!.addEventListener("click", async () => {
    await invoke("lock_vault_cmd");
    await showUnlock();
  });

  const secretTa = document.querySelector<HTMLTextAreaElement>(
    '#add-form textarea[name="secret"]',
  )!;
  const hintEl = document.querySelector<HTMLParagraphElement>("#shape-hint")!;
  secretTa.addEventListener("input", () => {
    const h = shapeHint(secretTa.value);
    hintEl.textContent = h || "";
  });

  document.querySelector("#add-form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const err = document.querySelector<HTMLParagraphElement>("#add-err")!;
    err.textContent = "";
    const fd = new FormData(e.target as HTMLFormElement);
    try {
      await invoke("add_entry_cmd", {
        id: crypto.randomUUID(),
        provider: String(fd.get("provider") || "") || null,
        alias: String(fd.get("alias")),
        tags: String(fd.get("tags") || "") || null,
        secret: String(fd.get("secret")),
        kind: null,
      });
      (e.target as HTMLFormElement).reset();
      hintEl.textContent = "";
      await reloadEntryTables();
    } catch (x) {
      err.textContent = String(x);
    }
  });

  document.querySelector("#password-add-form")!.addEventListener(
    "submit",
    async (e) => {
      e.preventDefault();
      const err =
        document.querySelector<HTMLParagraphElement>("#password-add-err")!;
      err.textContent = "";
      const fd = new FormData(e.target as HTMLFormElement);
      try {
        await invoke("add_entry_cmd", {
          id: crypto.randomUUID(),
          provider: null,
          alias: String(fd.get("alias")),
          tags: String(fd.get("tags") || "") || null,
          secret: String(fd.get("secret")),
          kind: "password",
        });
        (e.target as HTMLFormElement).reset();
        await reloadEntryTables();
      } catch (x) {
        err.textContent = String(x);
      }
    },
  );

  document.querySelector("#set-form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const err = document.querySelector<HTMLParagraphElement>("#set-err")!;
    err.textContent = "";
    const fd = new FormData(e.target as HTMLFormElement);
    try {
      await invoke("set_setting_cmd", {
        key: "idle_lock_minutes",
        value: String(fd.get("idle_lock_minutes") || "0"),
      });
      await invoke("set_setting_cmd", {
        key: "clear_clipboard_on_save",
        value: String(fd.get("clear_clipboard_on_save") || "0"),
      });
      await invoke("set_setting_cmd", {
        key: "clear_clipboard_after_copy_sec",
        value: String(fd.get("clear_clipboard_after_copy_sec") || "0"),
      });
      closeSettingsModal();
    } catch (x) {
      err.textContent = String(x);
    }
  });

  void ensurePendingCliSnippetFocusListener();
  schedulePendingCliSnippetPulls();
  queueMicrotask(() => flushQueuedCliSnippetPayload());
}

function escapeHtml(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/"/g, "&quot;");
}

function escapeAttr(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;");
}

async function showQuickSave() {
  rerenderCurrent = showQuickSave;
  let clip = "";
  try {
    clip = await invoke<string>("read_clipboard_cmd");
  } catch {
    /* empty */
  }
  render(`
    ${langSelectHtml()}
    <div class="page page-narrow">
      <h1 class="page-title">${escapeHtml(t("quickSaveTitle"))}</h1>
      <section class="panel">
        <form id="qs-form" class="stack">
          <label>${escapeHtml(t("quickSaveAlias"))}</label><input name="alias" required />
          <fieldset class="stack entry-kind-fieldset">
            <legend>${escapeHtml(t("quickSaveEntryKind"))}</legend>
            <label class="radio-row"><input type="radio" name="entry_kind" value="api" checked /> ${escapeHtml(t("quickSaveKindApi"))}</label>
            <label class="radio-row"><input type="radio" name="entry_kind" value="password" /> ${escapeHtml(t("quickSaveKindPassword"))}</label>
          </fieldset>
          <div id="qs-api-only" class="stack">
            <label>${escapeHtml(t("labelProviderPreset"))}</label>
            ${providerPresetSelectHtml()}
            <p class="field-hint">${escapeHtml(t("providerPresetHint"))}</p>
            <label>${escapeHtml(t("quickSaveProvider"))}</label><input name="provider" />
          </div>
          <label>${escapeHtml(t("quickSaveTags"))}</label><input name="tags" />
          <label>${escapeHtml(t("quickSaveSecret"))}</label><textarea name="secret" rows="4" required>${escapeHtml(clip)}</textarea>
          <p class="hint" id="qs-hint"></p>
          <div class="form-actions">
            <button type="submit">${escapeHtml(t("quickSaveSave"))}</button>
            <button type="button" class="secondary" id="qs-cancel">${escapeHtml(t("quickSaveCancel"))}</button>
          </div>
          <p class="err" id="qs-err"></p>
        </form>
      </section>
    </div>
  `);
  wireHeader();
  wireProviderPreset("#qs-form");
  const qsForm = document.getElementById("qs-form")!;
  const syncQsApiOnly = () => {
    const apiOnly = document.getElementById("qs-api-only");
    const isPw =
      qsForm.querySelector<HTMLInputElement>(
        'input[name="entry_kind"]:checked',
      )?.value === "password";
    if (apiOnly) apiOnly.toggleAttribute("hidden", isPw);
  };
  qsForm
    .querySelectorAll<HTMLInputElement>('input[name="entry_kind"]')
    .forEach((el) => el.addEventListener("change", syncQsApiOnly));
  syncQsApiOnly();
  const ta = document.querySelector<HTMLTextAreaElement>(
    '#qs-form textarea[name="secret"]',
  )!;
  const h = document.querySelector<HTMLParagraphElement>("#qs-hint")!;
  const upd = () => {
    const x = shapeHint(ta.value);
    h.textContent = x || "";
  };
  ta.addEventListener("input", upd);
  upd();

  document.querySelector("#qs-cancel")!.addEventListener("click", async () => {
    await getCurrentWindow().close();
  });

  document.querySelector("#qs-form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const err = document.querySelector<HTMLParagraphElement>("#qs-err")!;
    err.textContent = "";
    const fd = new FormData(e.target as HTMLFormElement);
    const secret = String(fd.get("secret"));
    const id = crypto.randomUUID();
    const kindRaw = String(fd.get("entry_kind") || "api");
    const isPassword = kindRaw === "password";
    try {
      await invoke("add_entry_cmd", {
        id,
        provider: isPassword
          ? null
          : String(fd.get("provider") || "") || null,
        alias: String(fd.get("alias")),
        tags: String(fd.get("tags") || "") || null,
        secret,
        kind: isPassword ? "password" : null,
      });
      const clear = await invoke<string | null>("get_setting_cmd", {
        key: "clear_clipboard_on_save",
      });
      if (clear === "1") {
        await invoke("write_clipboard_cmd", { text: "" });
      }
      await getCurrentWindow().close();
    } catch (x) {
      err.textContent = String(x);
    }
  });
}

async function showUnlock() {
  clearIdleLoop();
  rerenderCurrent = showUnlock;
  render(`
    ${langSelectHtml()}
    <div class="page page-narrow">
      <h1 class="page-title">${escapeHtml(t("unlockTitle"))}</h1>
      <section class="panel">
        <form id="unlock-form" class="stack">
          <label>${escapeHtml(t("unlockMasterPassword"))}</label>
          <input type="password" name="password" required autocomplete="current-password" />
          <button type="submit">${escapeHtml(t("unlockSubmit"))}</button>
          <p class="err" id="unlock-err"></p>
        </form>
      </section>
    </div>
  `);
  wireHeader();
  wireUnlock();
}

async function showInit(path: string) {
  clearIdleLoop();
  rerenderCurrent = () => showInit(path);
  render(`
    ${langSelectHtml()}
    <div class="page page-narrow">
      <h1 class="page-title">${escapeHtml(t("initTitle"))}</h1>
      <section class="panel">
        <p class="path-hint">${escapeHtml(tf("initHintNewVault", { path }))}</p>
        <form id="init-form" class="stack">
          <label>${escapeHtml(t("initMasterPassword"))}</label>
          <input type="password" name="p1" required autocomplete="new-password" />
          <label>${escapeHtml(t("initConfirm"))}</label>
          <input type="password" name="p2" required autocomplete="new-password" />
          <button type="submit">${escapeHtml(t("initSubmit"))}</button>
          <p class="err" id="init-err"></p>
        </form>
      </section>
    </div>
  `);
  wireHeader();
  wireInit();
}

async function presentQuickEntry() {
  const unlocked = await invoke<boolean>("is_unlocked_cmd");
  if (!unlocked) {
    rerenderCurrent = presentQuickEntry;
    render(`
      ${langSelectHtml()}
      <div class="page page-narrow">
        <h1 class="page-title">${escapeHtml(t("quickSaveTitle"))}</h1>
        <section class="panel">
          <p class="hint">${escapeHtml(t("miniUnlockHint"))}</p>
          <form id="mini-unlock" class="stack">
            <label>${escapeHtml(t("miniUnlockPassword"))}</label>
            <input type="password" name="password" required />
            <button type="submit">${escapeHtml(t("miniUnlockSubmit"))}</button>
            <p class="err" id="mu-err"></p>
          </form>
        </section>
      </div>
    `);
    wireHeader();
    document.querySelector("#mini-unlock")!.addEventListener("submit", async (e) => {
      e.preventDefault();
      const fd = new FormData(e.target as HTMLFormElement);
      const err = document.querySelector<HTMLParagraphElement>("#mu-err")!;
      err.textContent = "";
      try {
        await invoke("unlock_vault_cmd", {
          password: String(fd.get("password")),
        });
        await showQuickSave();
      } catch (x) {
        err.textContent = String(x);
      }
    });
    return;
  }
  await showQuickSave();
}

async function bootstrap() {
  const path = await defaultPath();
  const params = new URLSearchParams(window.location.search);
  if (params.get("quick") === "1") {
    await presentQuickEntry();
    return;
  }

  const initialized = await isVaultInitialized();
  if (!initialized) await showInit(path);
  else await showUnlock();
}

bootstrap().catch((e) => {
  appEl.innerHTML = `<div class="page page-narrow"><section class="panel"><p class="err">${escapeHtml(String(e))}</p></section></div>`;
});

void listen<string>("pending-cli-snippet", (e) => {
  void acceptCliSnippetPayload(e.payload);
});

listen("open-quick-save", async () => {
  const ok = await invoke<boolean>("is_unlocked_cmd");
  if (!ok) {
    await getCurrentWindow().setFocus();
    return;
  }
  const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
  new WebviewWindow(`quick-${Date.now()}`, {
    url: `/?quick=1`,
    title: windowTitleQuickSave(),
    width: 480,
    height: 580,
    resizable: true,
  });
});
