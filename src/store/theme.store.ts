// 职责：外观意图(mode) + 解析后的实际外观(resolved) + 持久化。
//       「自动跟随系统」= mode==='system'，故无需额外字段，避免与 settings 跨域耦合。
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

/** 亮 ⇄ 暗：以当前解析结果取反，mode 为 system 时也能一次切到相反外观。 */
const toggle = () => setMode(themeStore.resolved === "dark" ? "light" : "dark");

/** 由 useThemeEngine 在监听到系统变化时回写，供图标等读取。 */
const setResolved = (resolved: ResolvedTheme) =>
  setThemeStore("resolved", resolved);

export { themeStore, setMode, toggle, setResolved };
