# Keycard manual test checklist (DoD)

Use a **throwaway** vault and a distinctive test secret (e.g. `sk-manual-TEST-ONLY`) that you can grep for.

## DoD §2 — Quick save / tray / shortcut

- [ ] Tray: **Save clipboard…** opens quick-save; alias required; save creates a row in the main list.
- [ ] **⌘⇧K** / **Ctrl+Shift+K** opens quick-save when vault is unlocked; does not open (or focuses main) when locked.
- [ ] Shape hint appears for `sk-…` / `Bearer` text without auto-changing the secret field.

## DoD §7 — No secrets in logs / stderr / UI errors

- [ ] Wrong master password: dialog or stderr contains **no** substring of a known test secret.
- [ ] Corrupt `vault.db` or wrong file: errors contain **no** decrypted key material.
- [ ] Copy secret: optional timed clipboard clear works; no secret text in devtools console from error paths you trigger.

## DoD §8 — Regression smoke

- [ ] `cargo test -p keycard-core`
- [ ] `cargo test -p keycard-cli`
- [ ] `npm run tauri dev` launches, init → unlock → add entry → lock → unlock.
