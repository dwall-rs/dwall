// 职责：固定模式——统一/单独互斥、当前显示器、每显示器主题草稿、逐条「应用/停止」。
//       配置是唯一真值源：应用 = 把草稿写进 config.wallpaper_mode 并重启引擎。
import { createStore } from "solid-js/store";

import type { Config, WallpaperMode } from "@/domain/config";
import { configuredThemeId } from "@/domain/config";
import { catalogStore } from "./catalog.store";
import { applyWallpaperMode, settingsStore } from "./settings.store";

type StrMap = Record<string, string>;

interface FixedState {
  allUnified: boolean; // 统一所有显示器（与单独设置结构性互斥）
  curMon: string; // 当前编辑的显示器 id
  selIndex: number | null; // null = 预览当前匹配的那张
  monitorThemes: StrMap; // 草稿：monId -> themeId（"all" 表示统一）
}

const [fixedStore, setFixedStore] = createStore<FixedState>({
  allUnified: true,
  curMon: "all",
  selIndex: null,
  monitorThemes: {},
});

/** 当前作用域 key。 */
const scopeKey = (): string =>
  fixedStore.allUnified ? "all" : fixedStore.curMon;

/** 用配置初始化草稿（配置与显示器加载后调用）。 */
const syncFromConfig = (config: Config | null) => {
  const mode = config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return;

  const m = mode.monitor_specific_wallpapers;
  if (typeof m === "string") {
    setFixedStore({
      allUnified: true,
      curMon: "all",
      monitorThemes: { all: m },
    });
  } else {
    setFixedStore({
      allUnified: false,
      curMon: catalogStore.monitors[0]?.device_path ?? "all",
      monitorThemes: { ...m },
    });
  }
};

/** 某显示器当前生效的主题（草稿 → 配置 → 目录首个）。 */
const themeForMon = (monId: string): string | undefined =>
  fixedStore.monitorThemes[monId] ??
  configuredThemeId(settingsStore.config, monId) ??
  catalogStore.themes[0]?.id;

/** 当前作用域生效的主题（草稿 → 配置 → 目录首个）。 */
const currentThemeId = (): string | undefined => themeForMon(scopeKey());

/** 该作用域是否已写入配置且与当前选择一致。 */
const isApplied = (key: string): boolean => {
  const mode = settingsStore.config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return false;
  const theme =
    fixedStore.monitorThemes[key] ??
    configuredThemeId(settingsStore.config, key);
  if (!theme) return false;
  const m = mode.monitor_specific_wallpapers;
  return typeof m === "string" ? m === theme : m[key] === theme;
};

/** 应用/停止该作用域（写配置并重启引擎）。 */
const toggleApply = async (key: string) => {
  const config = settingsStore.config;
  if (!config) return;
  const theme = fixedStore.monitorThemes[key] ?? configuredThemeId(config, key);
  if (!theme) return;

  const mode = config.wallpaper_mode;
  const fixedMap =
    mode?.mode === "fixed" &&
    typeof mode.monitor_specific_wallpapers === "object"
      ? { ...mode.monitor_specific_wallpapers }
      : {};

  let nextMode: WallpaperMode;
  if (fixedStore.allUnified) {
    nextMode = {
      mode: "fixed",
      monitor_specific_wallpapers: isApplied(key) ? {} : theme,
    };
  } else {
    if (isApplied(key)) {
      delete fixedMap[key];
    } else {
      fixedMap[key] = theme;
    }
    nextMode = { mode: "fixed", monitor_specific_wallpapers: fixedMap };
  }

  await applyWallpaperMode(nextMode);
};

const toggleUnified = () => {
  setFixedStore((s) => ({
    ...s,
    allUnified: !s.allUnified,
    curMon: !s.allUnified ? "all" : s.curMon,
    selIndex: null,
  }));
};

const selectMon = (id: string) => {
  setFixedStore((s) => ({ ...s, curMon: id, selIndex: null }));
};

const selectWallpaper = (selIndex: number | null) => {
  setFixedStore("selIndex", selIndex);
};

const setMonitorTheme = (monId: string, themeId: string) => {
  setFixedStore("monitorThemes", (s) => ({ ...s, [monId]: themeId }));
  selectWallpaper(null);
};

export {
  fixedStore,
  scopeKey,
  themeForMon,
  currentThemeId,
  isApplied,
  toggleApply,
  toggleUnified,
  selectMon,
  selectWallpaper,
  setMonitorTheme,
  syncFromConfig,
};
