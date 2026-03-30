#!/usr/bin/env bash
# Receives selected text from a macOS Quick Action, writes it for Keycard to pick up,
# then activates the Keycard app. Install as a "Quick Action" → Services / right-click.
#
# Automator "Run Shell Script" must pass input as **stdin** for best results. If it is set to
# **as arguments** instead, we still pick up $* (space-joined); quoted paths may split wrong—use stdin.
set -euo pipefail
KEYCARD_DIR="${HOME}/Library/Application Support/Keycard"
mkdir -p "$KEYCARD_DIR"
OUT="${KEYCARD_DIR}/pending_cli_snippet.txt"

content=""
# When stdin is not a TTY (normal Quick Action), read the selection from the pipe.
if [ ! -t 0 ]; then
  content=$(cat || true)
fi
# Fallback: Automator often leaves stdin empty when "传递输入" is "作为自变量".
if [[ -z "${content//[[:space:]]/}" ]] && (($# > 0)); then
  content=$*
fi
printf '%s' "$content" > "$OUT"
# Bundle id from tauri.conf.json identifier (works with installed .app)
if ! osascript -e 'tell application id "app.keycard.desktop" to activate' 2>/dev/null; then
  osascript -e 'tell application "Keycard" to activate' 2>/dev/null || true
fi
