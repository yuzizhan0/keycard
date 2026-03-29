import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";

type EntryMeta = {
  id: string;
  provider: string | null;
  alias: string;
  tags: string | null;
  created_at: number;
};

const appEl = document.querySelector<HTMLDivElement>("#app")!;

function shapeHint(secret: string): string | null {
  const t = secret.trim();
  if (t.startsWith("sk-")) return "Looks like an OpenAI-style secret key (sk-…).";
  if (/^Bearer\s+/i.test(t)) return "Value starts with “Bearer”; you may want only the token part.";
  return null;
}

function render(html: string) {
  appEl.innerHTML = html;
}

async function defaultPath(): Promise<string> {
  return invoke<string>("default_vault_path_cmd");
}

async function needsInit(): Promise<boolean> {
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
        password_confirm: b,
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

async function idleLoop() {
  setInterval(async () => {
    const mins = await invoke<string | null>("get_setting_cmd", {
      key: "idle_lock_minutes",
    });
    const m = mins ? parseInt(mins, 10) : 0;
    if (!m || m <= 0) return;
    if (Date.now() - lastActivity > m * 60_000) {
      try {
        await invoke("lock_vault_cmd");
        await showUnlock();
      } catch {
        /* ignore */
      }
    }
  }, 15_000);
}

async function showMain() {
  const entries = await invoke<EntryMeta[]>("list_entries_json_cmd");
  const settings = await loadSettings();

  render(`
    <h1>Keycard</h1>
    <section>
      <div class="toolbar">
        <input type="search" id="q" placeholder="Search alias / provider…" />
        <button type="button" class="secondary" id="lock">Lock</button>
      </div>
      <table><thead><tr><th>Alias</th><th>Provider</th><th>Id</th><th></th></tr></thead><tbody id="rows"></tbody></table>
    </section>
    <section>
      <h2 style="font-size:1rem;margin:0 0 .5rem">Add entry</h2>
      <form id="add-form">
        <label>Id (unique)</label><input name="id" required />
        <label>Alias</label><input name="alias" required />
        <label>Provider</label><input name="provider" />
        <label>Tags</label><input name="tags" />
        <label>Secret</label><textarea name="secret" rows="3" required></textarea>
        <p class="hint" id="shape-hint"></p>
        <button type="submit">Save entry</button>
        <p class="err" id="add-err"></p>
      </form>
    </section>
    <section>
      <h2 style="font-size:1rem;margin:0 0 .5rem">Settings</h2>
      <form id="set-form">
        <label>Idle lock (minutes, 0 = off)</label>
        <input name="idle_lock_minutes" type="number" min="0" value="${settings.idle_lock_minutes ?? "15"}" />
        <label>Clear clipboard after successful quick-save</label>
        <select name="clear_clipboard_on_save">
          <option value="0" ${settings.clear_clipboard_on_save === "1" ? "" : "selected"}>No</option>
          <option value="1" ${settings.clear_clipboard_on_save === "1" ? "selected" : ""}>Yes</option>
        </select>
        <label>Clear clipboard N seconds after copying a secret (0 = off)</label>
        <input name="clear_clipboard_after_copy_sec" type="number" min="0" value="${settings.clear_clipboard_after_copy_sec ?? "0"}" />
        <button type="submit">Save settings</button>
        <p class="err" id="set-err"></p>
      </form>
    </section>
  `);

  idleLoop();

  const tbody = document.querySelector<HTMLTableSectionElement>("#rows")!;
  const q = document.querySelector<HTMLInputElement>("#q")!;

  function rowHtml(e: EntryMeta) {
    const prov = e.provider ?? "";
    const idShort =
      e.id.length > 12 ? `${e.id.slice(0, 6)}…${e.id.slice(-4)}` : e.id;
    return `<tr data-alias="${e.alias.toLowerCase()}" data-prov="${prov.toLowerCase()}">
      <td>${escapeHtml(e.alias)}</td><td>${escapeHtml(prov)}</td><td title="${escapeHtml(e.id)}">${escapeHtml(idShort)}</td>
      <td><button type="button" class="secondary copy" data-id="${escapeHtml(e.id)}">Copy</button></td>
    </tr>`;
  }

  function applyFilter() {
    const qq = q.value.trim().toLowerCase();
    tbody.querySelectorAll("tr").forEach((tr) => {
      const a = tr.getAttribute("data-alias") || "";
      const p = tr.getAttribute("data-prov") || "";
      tr.style.display = !qq || a.includes(qq) || p.includes(qq) ? "" : "none";
    });
  }

  function refreshTable(list: EntryMeta[]) {
    tbody.innerHTML = list.map(rowHtml).join("");
    tbody.querySelectorAll(".copy").forEach((btn) => {
      btn.addEventListener("click", async () => {
        const id = (btn as HTMLButtonElement).dataset.id!;
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
      });
    });
    applyFilter();
  }

  refreshTable(entries);
  q.addEventListener("input", applyFilter);

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
        id: String(fd.get("id")),
        provider: String(fd.get("provider") || "") || null,
        alias: String(fd.get("alias")),
        tags: String(fd.get("tags") || "") || null,
        secret: String(fd.get("secret")),
      });
      (e.target as HTMLFormElement).reset();
      hintEl.textContent = "";
      const list = await invoke<EntryMeta[]>("list_entries_json_cmd");
      refreshTable(list);
    } catch (x) {
      err.textContent = String(x);
    }
  });

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
    } catch (x) {
      err.textContent = String(x);
    }
  });
}

function escapeHtml(s: string) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/"/g, "&quot;");
}

async function showQuickSave() {
  let clip = "";
  try {
    clip = await invoke<string>("read_clipboard_cmd");
  } catch {
    /* empty */
  }
  render(`
    <h1>Quick save</h1>
    <section>
      <form id="qs-form">
        <label>Alias (required)</label><input name="alias" required />
        <label>Provider</label><input name="provider" />
        <label>Tags</label><input name="tags" />
        <label>Secret</label><textarea name="secret" rows="4" required>${escapeHtml(clip)}</textarea>
        <p class="hint" id="qs-hint"></p>
        <button type="submit">Save</button>
        <button type="button" class="secondary" id="qs-cancel">Cancel</button>
        <p class="err" id="qs-err"></p>
      </form>
    </section>
  `);
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
    try {
      await invoke("add_entry_cmd", {
        id,
        provider: String(fd.get("provider") || "") || null,
        alias: String(fd.get("alias")),
        tags: String(fd.get("tags") || "") || null,
        secret,
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
  render(`
    <h1>Unlock Keycard</h1>
    <section>
      <form id="unlock-form">
        <label>Master password</label>
        <input type="password" name="password" required autocomplete="current-password" />
        <button type="submit">Unlock</button>
        <p class="err" id="unlock-err"></p>
      </form>
    </section>
  `);
  wireUnlock();
}

async function showInit(path: string) {
  render(`
    <h1>Create vault</h1>
    <section>
      <p class="hint">New vault at: ${escapeHtml(path)}</p>
      <form id="init-form">
        <label>Master password</label>
        <input type="password" name="p1" required autocomplete="new-password" />
        <label>Confirm</label>
        <input type="password" name="p2" required autocomplete="new-password" />
        <button type="submit">Create</button>
        <p class="err" id="init-err"></p>
      </form>
    </section>
  `);
  wireInit();
}

async function bootstrap() {
  const path = await defaultPath();
  const params = new URLSearchParams(window.location.search);
  if (params.get("quick") === "1") {
    const unlocked = await invoke<boolean>("is_unlocked_cmd");
    if (!unlocked) {
      render(`<h1>Quick save</h1><section><p class="hint">Enter master password for this vault.</p>
        <form id="mini-unlock"><label>Password</label><input type="password" name="password" required /><button type="submit">Unlock</button><p class="err" id="mu-err"></p></form></section>`);
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
    return;
  }

  const init = await needsInit();
  if (init) await showInit(path);
  else await showUnlock(path);
}

bootstrap().catch((e) => {
  appEl.innerHTML = `<p class="err">${escapeHtml(String(e))}</p>`;
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
    title: "Keycard — Quick save",
    width: 480,
    height: 520,
    resizable: true,
  });
});
