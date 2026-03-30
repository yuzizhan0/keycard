/** Light / dark appearance; persisted like locale. */

export type Theme = "light" | "dark";

const STORAGE_KEY = "keycard_theme";

const SUN_ICON = `<svg class="theme-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><circle cx="12" cy="12" r="4" stroke="currentColor" stroke-width="1.5"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/></svg>`;

const MOON_ICON = `<svg class="theme-svg" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg" aria-hidden="true"><path d="M21 12.79A9 9 0 1111.21 3 7 7 0 0021 12.79z" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>`;

export function getTheme(): Theme {
  try {
    const s = localStorage.getItem(STORAGE_KEY);
    if (s === "light" || s === "dark") return s;
  } catch {
    /* private mode */
  }
  return "dark";
}

export function setTheme(theme: Theme): void {
  document.documentElement.setAttribute("data-theme", theme);
  document.documentElement.style.colorScheme =
    theme === "light" ? "light" : "dark";
  try {
    localStorage.setItem(STORAGE_KEY, theme);
  } catch {
    /* ignore */
  }
  const meta = document.querySelector('meta[name="theme-color"]');
  meta?.setAttribute(
    "content",
    theme === "light" ? "#eef0f5" : "#07070c",
  );
}

export function initTheme(): void {
  setTheme(getTheme());
}

export function toggleTheme(): void {
  setTheme(getTheme() === "dark" ? "light" : "dark");
}

/** Icon meaning: in dark mode show sun (switch to light); in light mode show moon (switch to dark). */
export function themeToggleInnerHtml(): string {
  return getTheme() === "dark" ? SUN_ICON : MOON_ICON;
}
