// Responsibility: fixed mode — unified/per-monitor exclusivity, current monitor, per-monitor theme drafts,
//               per-entry "apply/stop". Config is the source of truth: apply = write the draft into config.wallpaper_mode and restart the engine.
import { createStore } from "solid-js/store";

import type { Config, WallpaperMode } from "@/domain/config";
import { configuredThemeId } from "@/domain/config";
import { catalogStore } from "./catalog.store";
import { engineStore } from "./engine.store";
import { applyWallpaperMode, settingsStore } from "./settings.store";

type StrMap = Record<string, string>;

interface FixedState {
  allUnified: boolean; // unify all monitors (structurally mutually exclusive with per-monitor settings)
  curMon: string; // id of the monitor being edited
  selIndex: number | null; // null = preview the currently matching entry
  monitorThemes: StrMap; // draft: monId -> themeId ("all" means unified)
}

const [fixedStore, setFixedStore] = createStore<FixedState>({
  allUnified: true,
  curMon: "all",
  selIndex: null,
  monitorThemes: {},
});

/** The current scope key. */
const scopeKey = (): string =>
  fixedStore.allUnified ? "all" : fixedStore.curMon;

/** Seed the draft from the config (called once config and monitors are loaded). */
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

/** Theme in effect for a monitor (draft → config → first entry in the catalog). */
const themeForMon = (monId: string): string | undefined =>
  fixedStore.monitorThemes[monId] ??
  configuredThemeId(settingsStore.config, monId) ??
  catalogStore.themes[0]?.id;

/** Theme in effect for the current scope (draft → config → first entry in the catalog). */
const currentThemeId = (): string | undefined => themeForMon(scopeKey());

/** Whether this scope has been written into the config (regardless of whether the engine is running). */
const isConfigured = (key: string): boolean => {
  const mode = settingsStore.config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return false;
  const theme =
    fixedStore.monitorThemes[key] ??
    configuredThemeId(settingsStore.config, key);
  if (!theme) return false;
  const m = mode.monitor_specific_wallpapers;
  return typeof m === "string" ? m === theme : m[key] === theme;
};

/** Whether this scope is actually in effect right now: written into the config and the engine is running. */
const isApplied = (key: string): boolean =>
  engineStore.running && isConfigured(key);

/** Apply/stop that scope (write the config and restart the engine). */
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
    // When unified is turned off, select the first monitor so the scope does not stay on the pseudo-monitor "all".
    curMon: s.allUnified
      ? (catalogStore.monitors[0]?.device_path ?? "all")
      : "all",
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
