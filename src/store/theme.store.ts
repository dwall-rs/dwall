// Responsibility: appearance intent (mode) + resolved appearance (resolved) + persistence.
//       "Auto follow system" = mode==='system'; no extra field is needed, avoiding cross-domain coupling with settings.
import type { ResolvedTheme, ThemeMode } from "@/domain/types";
import { createStore } from "solid-js/store";

const STORAGE_KEY = "theme-mode";

function readStoredMode(): ThemeMode {
  if (typeof window === "undefined") return "system";
  const v = window.localStorage.getItem(STORAGE_KEY);
  return v === "light" || v === "dark" || v === "system" ? v : "system";
}

function computeResolved(mode: ThemeMode): ResolvedTheme {
  if (mode !== "system") return mode;
  return typeof window !== "undefined" &&
    window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

interface ThemeState {
  mode: ThemeMode;
  resolved: ResolvedTheme;
}

const [themeStore, setThemeStore] = createStore<ThemeState>({
  mode: readStoredMode(),
  resolved: computeResolved(readStoredMode()),
});

const setMode = (mode: ThemeMode) => {
  try {
    window.localStorage.setItem(STORAGE_KEY, mode);
  } catch {
    /* ignore */
  }
  setThemeStore({ mode, resolved: computeResolved(mode) });
};

/** Light ⇄ dark: invert the current resolved value, so one call flips to the opposite appearance even in system mode. */
const toggle = () => setMode(themeStore.resolved === "dark" ? "light" : "dark");

/** Written back by useThemeEngine when a system change is detected, for icons and others to read. */
const setResolved = (resolved: ResolvedTheme) =>
  setThemeStore("resolved", resolved);

export { themeStore, setMode, toggle, setResolved };
