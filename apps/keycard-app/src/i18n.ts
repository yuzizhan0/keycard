/** UI locales: English, 简体中文, 日本語, 한국어 */

import { PROVIDER_PRESET_ROWS } from "./provider-presets";
import { getTheme, themeToggleInnerHtml } from "./theme";

export type Locale = "en" | "zh" | "ja" | "ko";

export const LOCALE_LABELS: Record<Locale, string> = {
  en: "English",
  zh: "简体中文",
  ja: "日本語",
  ko: "한국어",
};

const STORAGE_KEY = "keycard_locale";

type Msg = {
  langLabel: string;
  unlockTitle: string;
  unlockMasterPassword: string;
  unlockSubmit: string;
  initTitle: string;
  initHintNewVault: string;
  initMasterPassword: string;
  initConfirm: string;
  initSubmit: string;
  brandName: string;
  mainTitle: string;
  mainSearchPlaceholder: string;
  mainLock: string;
  mainTabSecrets: string;
  mainTabCli: string;
  mainTabsAria: string;
  mainColAlias: string;
  mainColProvider: string;
  mainColActions: string;
  mainAddEntry: string;
  labelAlias: string;
  labelProvider: string;
  labelProviderPreset: string;
  providerPresetPlaceholder: string;
  providerPresetHint: string;
  labelTags: string;
  labelSecret: string;
  mainSaveEntry: string;
  mainSettings: string;
  mainIdleLock: string;
  mainClearClipQuickSave: string;
  mainYes: string;
  mainNo: string;
  mainClearClipAfterCopy: string;
  mainSaveSettings: string;
  mainCopy: string;
  shapeOpenai: string;
  shapeBearer: string;
  quickSaveTitle: string;
  quickSaveAlias: string;
  quickSaveProvider: string;
  quickSaveTags: string;
  quickSaveSecret: string;
  quickSaveSave: string;
  quickSaveCancel: string;
  miniUnlockHint: string;
  miniUnlockPassword: string;
  miniUnlockSubmit: string;
  windowQuickSaveTitle: string;
  settingsCloseAria: string;
  themeSwitchToLight: string;
  themeSwitchToDark: string;
  cliSectionTitle: string;
  cliSectionHint: string;
  cliProfileHint: string;
  cliTerminalHint: string;
  cliColName: string;
  cliColProfile: string;
  cliColCommand: string;
  cliColActions: string;
  /** Clipboard: same command line as stored (program + argv, quoted if needed) */
  cliCopyArgv: string;
  /** Tooltip: this copies the real npm/script line */
  cliCopyArgvTitle: string;
  cliDelete: string;
  cliSaveCommand: string;
  labelCliName: string;
  labelCliProfile: string;
  cliProfileNone: string;
  labelCliProgram: string;
  labelCliArgs: string;
  labelCliArgsHint: string;
  labelCliNotes: string;
  cliErrProgram: string;
};

const en: Msg = {
  langLabel: "Language",
  unlockTitle: "Unlock Keycard",
  unlockMasterPassword: "Master password",
  unlockSubmit: "Unlock",
  initTitle: "Create vault",
  initHintNewVault: "New vault at: {path}",
  initMasterPassword: "Master password",
  initConfirm: "Confirm",
  initSubmit: "Create",
  brandName: "Keycard",
  mainTitle: "Keycard",
  mainSearchPlaceholder: "Search name, provider, tags…",
  mainLock: "Lock",
  mainTabSecrets: "Model keys",
  mainTabCli: "CLI commands",
  mainTabsAria: "Main sections",
  mainColAlias: "Name",
  mainColProvider: "Provider",
  mainColActions: "",
  mainAddEntry: "Add entry",
  labelAlias: "Name",
  labelProvider: "Provider",
  labelProviderPreset: "Popular model / provider",
  providerPresetPlaceholder: "Quick pick…",
  providerPresetHint:
    "Choosing a preset fills “Provider” below; you can still edit it.",
  labelTags: "Tags",
  labelSecret: "Secret",
  mainSaveEntry: "Save entry",
  mainSettings: "Settings",
  mainIdleLock: "Idle lock (minutes, 0 = off)",
  mainClearClipQuickSave: "Clear clipboard after successful quick-save",
  mainYes: "Yes",
  mainNo: "No",
  mainClearClipAfterCopy:
    "Clear clipboard N seconds after copying a secret (0 = off)",
  mainSaveSettings: "Save settings",
  mainCopy: "Copy",
  shapeOpenai: "Looks like an OpenAI-style secret key (sk-…).",
  shapeBearer:
    "Value starts with “Bearer”; you may want only the token part.",
  quickSaveTitle: "Quick save",
  quickSaveAlias: "Name (required)",
  quickSaveProvider: "Provider",
  quickSaveTags: "Tags",
  quickSaveSecret: "Secret",
  quickSaveSave: "Save",
  quickSaveCancel: "Cancel",
  miniUnlockHint: "Enter master password for this vault.",
  miniUnlockPassword: "Password",
  miniUnlockSubmit: "Unlock",
  windowQuickSaveTitle: "Keycard — Quick save",
  settingsCloseAria: "Close settings",
  themeSwitchToLight: "Switch to light mode",
  themeSwitchToDark: "Switch to dark mode",
  cliSectionTitle: "Saved CLI commands",
  cliSectionHint:
    "Run in the terminal: keycard saved run <name>. An optional profile injects vault env vars (same as keycard run -p).",
  cliProfileHint:
    "Profiles come from your vault (profiles table). If the list is empty, use “None” or add profiles outside the app for now.",
  cliTerminalHint:
    "macOS: select text in Terminal → right-click → Services → “Save CLI to Keycard” (after you add the Quick Action from scripts/macos; see README). Unlock Keycard first; the first non-empty line fills Program/Arguments (quotes group args; a leading `$`/`#` prompt is stripped when it looks like a shell prompt).",
  cliColName: "Name",
  cliColProfile: "Profile",
  cliColCommand: "Command",
  cliColActions: "",
  cliCopyArgv: "Copy command",
  cliCopyArgvTitle:
    "Copies the saved command line (what you selected, e.g. npm run …).",
  cliDelete: "Delete",
  cliSaveCommand: "Save command",
  labelCliName: "Short name",
  labelCliProfile: "Profile (optional)",
  cliProfileNone: "None",
  labelCliProgram: "Program",
  labelCliArgs: "Arguments",
  labelCliArgsHint:
    "Space-separated; use quotes for args that contain spaces. Example: build --release or run \"tauri dev\"",
  labelCliNotes: "Notes (optional)",
  cliErrProgram: "Program is required.",
};

const zh: Msg = {
  langLabel: "语言",
  unlockTitle: "解锁奇卡",
  unlockMasterPassword: "主密码",
  unlockSubmit: "解锁",
  initTitle: "创建保险库",
  initHintNewVault: "新保险库路径：{path}",
  initMasterPassword: "主密码",
  initConfirm: "确认密码",
  initSubmit: "创建",
  brandName: "奇卡",
  mainTitle: "奇卡",
  mainSearchPlaceholder: "搜索名称、服务商、标签…",
  mainLock: "锁定",
  mainTabSecrets: "模型密钥",
  mainTabCli: "cli指令",
  mainTabsAria: "主界面分区",
  mainColAlias: "名称",
  mainColProvider: "服务商",
  mainColActions: "",
  mainAddEntry: "添加条目",
  labelAlias: "名称",
  labelProvider: "服务商",
  labelProviderPreset: "主流大模型 / 服务商",
  providerPresetPlaceholder: "快捷选择…",
  providerPresetHint: "选择后会填入下方「服务商」，仍可手动修改。",
  labelTags: "标签",
  labelSecret: "密钥",
  mainSaveEntry: "保存条目",
  mainSettings: "设置",
  mainIdleLock: "空闲自动锁定（分钟，0 为关闭）",
  mainClearClipQuickSave: "快速保存成功后清空剪贴板",
  mainYes: "是",
  mainNo: "否",
  mainClearClipAfterCopy: "复制密钥后 N 秒清空剪贴板（0 为关闭）",
  mainSaveSettings: "保存设置",
  mainCopy: "复制",
  shapeOpenai: "看起来像 OpenAI 风格的密钥（sk-…）。",
  shapeBearer: "内容以 “Bearer” 开头；通常只需要其中的 token 部分。",
  quickSaveTitle: "快速保存",
  quickSaveAlias: "名称（必填）",
  quickSaveProvider: "服务商",
  quickSaveTags: "标签",
  quickSaveSecret: "密钥",
  quickSaveSave: "保存",
  quickSaveCancel: "取消",
  miniUnlockHint: "请输入此保险库的主密码。",
  miniUnlockPassword: "密码",
  miniUnlockSubmit: "解锁",
  windowQuickSaveTitle: "奇卡 — 快速保存",
  settingsCloseAria: "关闭设置",
  themeSwitchToLight: "切换到白天模式",
  themeSwitchToDark: "切换到夜间模式",
  cliSectionTitle: "常用 CLI 指令",
  cliSectionHint:
    "在终端执行：keycard saved run <名称>。可选 Profile 会注入保险库里的环境变量（与 keycard run -p 相同）。",
  cliProfileHint:
    "Profile 来自保险库（profiles 表）。若列表为空，可选「无」或先在库外配置 Profile。",
  cliTerminalHint:
    "macOS：在终端划选 → 右键 →「服务」→「Save CLI to Keycard」（需先在「自动操作」里添加，见 README）。请先解锁奇卡；取第一行非空拆成「程序」与「参数」（引号内算一个参数；行首像 `$`、`#` 的提示符会尽量去掉）。",
  cliColName: "名称",
  cliColProfile: "Profile",
  cliColCommand: "命令",
  cliColActions: "",
  cliCopyArgv: "复制命令",
  cliCopyArgvTitle: "复制已保存的整行命令（与划选的 npm run … 一致）。",
  cliDelete: "删除",
  cliSaveCommand: "保存指令",
  labelCliName: "简短名称",
  labelCliProfile: "Profile（可选）",
  cliProfileNone: "无",
  labelCliProgram: "程序",
  labelCliArgs: "参数",
  labelCliArgsHint:
    "按空格分段；含空格的参数请加引号。例：build --release 或 run \"tauri dev\"",
  labelCliNotes: "备注（可选）",
  cliErrProgram: "请填写程序名。",
};

const ja: Msg = {
  langLabel: "言語",
  unlockTitle: "Keycard のロック解除",
  unlockMasterPassword: "マスターパスワード",
  unlockSubmit: "ロック解除",
  initTitle: "ボールトを作成",
  initHintNewVault: "新しいボールトの場所: {path}",
  initMasterPassword: "マスターパスワード",
  initConfirm: "確認",
  initSubmit: "作成",
  brandName: "Keycard",
  mainTitle: "Keycard",
  mainSearchPlaceholder: "名前、プロバイダー、タグを検索…",
  mainLock: "ロック",
  mainTabSecrets: "モデルキー",
  mainTabCli: "CLI コマンド",
  mainTabsAria: "メインの切り替え",
  mainColAlias: "名前",
  mainColProvider: "プロバイダー",
  mainColActions: "",
  mainAddEntry: "エントリを追加",
  labelAlias: "名前",
  labelProvider: "プロバイダー",
  labelProviderPreset: "主要モデル / プロバイダー",
  providerPresetPlaceholder: "よく使うものから選択…",
  providerPresetHint:
    "選ぶと下の「プロバイダー」に入力されます。あとから編集できます。",
  labelTags: "タグ",
  labelSecret: "シークレット",
  mainSaveEntry: "エントリを保存",
  mainSettings: "設定",
  mainIdleLock: "アイドル時の自動ロック（分、0 でオフ）",
  mainClearClipQuickSave: "クイック保存成功後にクリップボードを消去",
  mainYes: "はい",
  mainNo: "いいえ",
  mainClearClipAfterCopy:
    "シークレットコピー後 N 秒でクリップボードを消去（0 でオフ）",
  mainSaveSettings: "設定を保存",
  mainCopy: "コピー",
  shapeOpenai: "OpenAI 形式のシークレットキー（sk-…）のようです。",
  shapeBearer:
    "「Bearer」で始まっています。トークン部分だけを保存することをおすすめします。",
  quickSaveTitle: "クイック保存",
  quickSaveAlias: "名前（必須）",
  quickSaveProvider: "プロバイダー",
  quickSaveTags: "タグ",
  quickSaveSecret: "シークレット",
  quickSaveSave: "保存",
  quickSaveCancel: "キャンセル",
  miniUnlockHint: "このボールトのマスターパスワードを入力してください。",
  miniUnlockPassword: "パスワード",
  miniUnlockSubmit: "ロック解除",
  windowQuickSaveTitle: "Keycard — クイック保存",
  settingsCloseAria: "設定を閉じる",
  themeSwitchToLight: "ライトモードに切り替え",
  themeSwitchToDark: "ダークモードに切り替え",
  cliSectionTitle: "保存した CLI コマンド",
  cliSectionHint:
    "ターミナルで keycard saved run <名前>。プロファイルを選ぶと vault の環境変数を注入します（keycard run -p と同様）。",
  cliProfileHint:
    "プロファイルは vault の profiles テーブル由来です。空なら「なし」か外部で追加してください。",
  cliTerminalHint:
    "macOS: ターミナルで選択 → 右クリック → サービス →「Save CLI to Keycard」（クイックアクション追加が必要、README 参照）。先に解除。最初の非空行を分割（引用符でまとまり、`$`/`#` プロンプトは可能なら除去）。",
  cliColName: "名前",
  cliColProfile: "プロファイル",
  cliColCommand: "コマンド",
  cliColActions: "",
  cliCopyArgv: "コマンドをコピー",
  cliCopyArgvTitle: "保存したコマンド行をコピー（選択した npm run など）。",
  cliDelete: "削除",
  cliSaveCommand: "保存",
  labelCliName: "短い名前",
  labelCliProfile: "プロファイル（任意）",
  cliProfileNone: "なし",
  labelCliProgram: "プログラム",
  labelCliArgs: "引数",
  labelCliArgsHint:
    "スペース区切り。空白を含む引数は引用符で囲む。例: build --release または run \"tauri dev\"",
  labelCliNotes: "メモ（任意）",
  cliErrProgram: "プログラムを入力してください。",
};

const ko: Msg = {
  langLabel: "언어",
  unlockTitle: "Keycard 잠금 해제",
  unlockMasterPassword: "마스터 비밀번호",
  unlockSubmit: "잠금 해제",
  initTitle: "금고 만들기",
  initHintNewVault: "새 금고 위치: {path}",
  initMasterPassword: "마스터 비밀번호",
  initConfirm: "확인",
  initSubmit: "만들기",
  brandName: "Keycard",
  mainTitle: "Keycard",
  mainSearchPlaceholder: "이름, 제공자, 태그 검색…",
  mainLock: "잠금",
  mainTabSecrets: "모델 키",
  mainTabCli: "CLI 명령",
  mainTabsAria: "메인 탭",
  mainColAlias: "이름",
  mainColProvider: "제공자",
  mainColActions: "",
  mainAddEntry: "항목 추가",
  labelAlias: "이름",
  labelProvider: "제공자",
  labelProviderPreset: "주요 모델 / 제공자",
  providerPresetPlaceholder: "빠른 선택…",
  providerPresetHint:
    "선택 시 아래「제공자」칸이 채워지며, 직접 수정할 수 있습니다.",
  labelTags: "태그",
  labelSecret: "비밀 값",
  mainSaveEntry: "항목 저장",
  mainSettings: "설정",
  mainIdleLock: "유휴 시 자동 잠금(분, 0이면 끔)",
  mainClearClipQuickSave: "빠른 저장 성공 후 클립보드 비우기",
  mainYes: "예",
  mainNo: "아니오",
  mainClearClipAfterCopy: "비밀 복사 후 N초 뒤 클립보드 비우기(0이면 끔)",
  mainSaveSettings: "설정 저장",
  mainCopy: "복사",
  shapeOpenai: "OpenAI 스타일 비밀 키(sk-…)로 보입니다.",
  shapeBearer:
    "값이 “Bearer”로 시작합니다. 토큰 부분만 저장하는 것이 좋습니다.",
  quickSaveTitle: "빠른 저장",
  quickSaveAlias: "이름(필수)",
  quickSaveProvider: "제공자",
  quickSaveTags: "태그",
  quickSaveSecret: "비밀 값",
  quickSaveSave: "저장",
  quickSaveCancel: "취소",
  miniUnlockHint: "이 금고의 마스터 비밀번호를 입력하세요.",
  miniUnlockPassword: "비밀번호",
  miniUnlockSubmit: "잠금 해제",
  windowQuickSaveTitle: "Keycard — 빠른 저장",
  settingsCloseAria: "설정 닫기",
  themeSwitchToLight: "라이트 모드로 전환",
  themeSwitchToDark: "다크 모드로 전환",
  cliSectionTitle: "저장된 CLI 명령",
  cliSectionHint:
    "터미널에서 keycard saved run <이름> 실행. 프로필을 선택하면 금고 환경 변수를 주입합니다(keycard run -p와 동일).",
  cliProfileHint:
    "프로필은 vault의 profiles 테이블에서 옵니다. 비어 있으면 「없음」을 선택하거나 외부에서 추가하세요.",
  cliTerminalHint:
    "macOS: 터미널에서 선택 → 오른쪽 클릭 → 서비스 →「Save CLI to Keycard」(빠른 동작 추가 필요, README 참고). 먼저 잠금 해제. 첫 비어 있지 않은 줄을 분할(따옴표로 묶음, `$`/`#` 프롬프트는 가능하면 제거).",
  cliColName: "이름",
  cliColProfile: "프로필",
  cliColCommand: "명령",
  cliColActions: "",
  cliCopyArgv: "명령 복사",
  cliCopyArgvTitle: "저장한 명령 줄 복사(선택한 npm run 등).",
  cliDelete: "삭제",
  cliSaveCommand: "저장",
  labelCliName: "짧은 이름",
  labelCliProfile: "프로필(선택)",
  cliProfileNone: "없음",
  labelCliProgram: "프로그램",
  labelCliArgs: "인수",
  labelCliArgsHint:
    "공백으로 구분. 공백이 있는 인수는 따옴표로 감싸기. 예: build --release 또는 run \"tauri dev\"",
  labelCliNotes: "메모(선택)",
  cliErrProgram: "프로그램을 입력하세요.",
};

const STRINGS: Record<Locale, Msg> = { en, zh, ja, ko };

function detectLocale(): Locale {
  try {
    const stored = localStorage.getItem(STORAGE_KEY) as Locale | null;
    if (stored && STRINGS[stored]) return stored;
  } catch {
    /* private mode */
  }
  const nav = navigator.language.toLowerCase();
  if (nav.startsWith("zh")) return "zh";
  if (nav.startsWith("ja")) return "ja";
  if (nav.startsWith("ko")) return "ko";
  return "en";
}

let current: Locale = detectLocale();
document.documentElement.lang = current === "zh" ? "zh-CN" : current;

export function getLocale(): Locale {
  return current;
}

export function setLocale(l: Locale): void {
  if (!STRINGS[l]) return;
  current = l;
  document.documentElement.lang = l === "zh" ? "zh-CN" : l;
  try {
    localStorage.setItem(STORAGE_KEY, l);
  } catch {
    /* ignore */
  }
}

export type MessageKey = keyof Msg;

export function t(key: MessageKey): string {
  const pack = STRINGS[current] ?? STRINGS.en;
  return pack[key] ?? STRINGS.en[key] ?? key;
}

export function tf(key: MessageKey, vars: Record<string, string>): string {
  let s = t(key);
  for (const [k, v] of Object.entries(vars)) {
    s = s.replaceAll(`{${k}}`, v);
  }
  return s;
}

const GEAR_ICON = `<svg class="gear-icon" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><path d="M12 15a3 3 0 100-6 3 3 0 000 6z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/><path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 010 2.83 2 2 0 01-2.83 0l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-2 2 2 2 0 01-2-2v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83 0 2 2 0 010-2.83l.06-.06a1.65 1.65 0 00.33-1.82 1.65 1.65 0 00-1.51-1H3a2 2 0 01-2-2 2 2 0 012-2h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 010-2.83 2 2 0 012.83 0l.06.06A1.65 1.65 0 009 4.6a1.65 1.65 0 001-1.51V3a2 2 0 012-2 2 2 0 012 2v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 0 2 2 0 010 2.83l-.06.06a1.65 1.65 0 00-.33 1.82V9a1.65 1.65 0 001.51 1H21a2 2 0 012 2 2 2 0 01-2 2h-.09a1.65 1.65 0 00-1.51 1z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

export type LangSelectOptions = {
  /** Main vault screen only: gear opens settings modal. */
  showSettingsGear?: boolean;
};

export function langSelectHtml(options?: LangSelectOptions): string {
  const cur = getLocale();
  const opts = (Object.keys(STRINGS) as Locale[])
    .map(
      (l) =>
        `<option value="${l}" ${l === cur ? "selected" : ""}>${LOCALE_LABELS[l]}</option>`,
    )
    .join("");
  const gearBtn = options?.showSettingsGear
    ? `<button type="button" class="header-icon-btn secondary" id="open-settings" aria-label="${escapeAttr(t("mainSettings"))}">${GEAR_ICON}</button>`
    : "";
  const themeAria =
    getTheme() === "dark" ? t("themeSwitchToLight") : t("themeSwitchToDark");
  const themeBtn = `<button type="button" class="header-icon-btn secondary" id="theme-toggle" aria-label="${escapeAttr(themeAria)}"><span class="theme-icon">${themeToggleInnerHtml()}</span></button>`;
  const brandImg = `<img class="brand-icon" src="/logo.svg" width="32" height="32" alt="" decoding="async" />`;
  return `<header class="app-header">
    <div class="brand">
      ${brandImg}
      <span class="brand-name">${escapeAttr(t("brandName"))}</span>
    </div>
    <div class="app-header-actions">
      ${themeBtn}
      ${gearBtn}
      <div class="lang-bar">
        <label for="lang-select">${escapeAttr(t("langLabel"))}</label>
        <select id="lang-select" class="lang-select">${opts}</select>
      </div>
    </div>
  </header>`;
}

function escapeAttr(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;");
}

function escOptionText(s: string): string {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

/** `<select id="provider-preset">` with localized labels; values are canonical English provider strings. */
export function providerPresetSelectHtml(): string {
  const loc = getLocale();
  const body = PROVIDER_PRESET_ROWS.map((row) => {
    const label =
      (row as Record<Locale, string>)[loc] ?? row.en;
    return `<option value="${escapeAttr(row.value)}">${escOptionText(label)}</option>`;
  }).join("");
  return `<select id="provider-preset" class="preset-select" aria-label="${escapeAttr(t("labelProviderPreset"))}"><option value="">${escOptionText(t("providerPresetPlaceholder"))}</option>${body}</select>`;
}

export function windowTitleQuickSave(): string {
  return STRINGS[getLocale()].windowQuickSaveTitle;
}
