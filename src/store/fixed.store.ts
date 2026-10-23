// Responsibility: fixed mode — unified/per-monitor exclusivity, current monitor, per-monitor theme drafts,
//               per-entry "apply/stop". Config is the source of truth: apply = write the draft into config.wallpaper_mode and restart the engine.
import { createStore } from "solid-js/store";

import { resolveMonitorThemes } from "@/ipc";
import type { MonitorTheme, MonitorThemes } from "@/ipc";
import type { Config } from "@/models/config";
import { logger } from "@/utils";
import { catalogStore } from "./catalog.store";
import { engineStore } from "./engine.store";
import { applyWallpaperMode, settingsStore } from "./settings.store";

const log = logger.child("fixed");

type StrMap = Record<string, string>;

interface FixedState {
  allUnified: boolean; // unify all monitors (structurally mutually exclusive with per-monitor settings)
  curMon: string; // id of the monitor being edited
  selIndex: number | null; // null = preview the currently matching entry
  monitorThemes: StrMap; // draft: monId -> themeId ("all" means unified)
  configThemes: StrMap; // effective themes of the config, "all" expanded
  configUnified: boolean; // whether the config stores one theme for every monitor
}

const [fixedStore, setFixedStore] = createStore<FixedState>({
  allUnified: true,
  curMon: "all",
  selIndex: null,
  monitorThemes: {},
  configThemes: {},
  configUnified: false,
});

/** The current scope key. */
const scopeKey = (): string =>
  fixedStore.allUnified ? "all" : fixedStore.curMon;

/** Device paths of the monitors Rust reported, in its order. */
const monitorIds = (): string[] =>
  catalogStore.monitors.map((m) => m.device_path);

const toRecord = (themes: MonitorTheme[]): StrMap =>
  Object.fromEntries(
    themes.map(({ monitorId, themeId }) => [monitorId, themeId]),
  );

/** The config's effective themes; random mode has no monitor mapping. */
const configuredThemes = (): StrMap =>
  settingsStore.config?.wallpaper_mode?.mode === "fixed"
    ? fixedStore.configThemes
    : {};

/** Seed the draft from the config (called once config and monitors are loaded). */
const syncFromConfig = async (config: Config | null) => {
  const mode = config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return;

  try {
    const { unified, themes, configured } = await resolveMonitorThemes(
      mode,
      monitorIds(),
    );
    const resolved = toRecord(themes);

    setFixedStore({
      allUnified: unified,
      curMon: unified
        ? "all"
        : (catalogStore.monitors[0]?.device_path ?? "all"),
      // The draft is the same shape it has always been seeded in: the unified
      // pseudo-monitor alone, or the entries the config actually stores.
      monitorThemes: unified
        ? { all: resolved.all }
        : Object.fromEntries(configured.map((id) => [id, resolved[id]])),
      configThemes: resolved,
      configUnified: unified,
    });
  } catch (e) {
    log.error("Failed to resolve the monitor themes", e);
  }
};

/** Theme in effect for a monitor (draft → config → first entry in the catalog). */
const themeForMon = (monId: string): string | undefined =>
  fixedStore.monitorThemes[monId] ??
  configuredThemes()[monId] ??
  catalogStore.themes[0]?.id;

/** Whether this scope has been written into the config (regardless of whether the engine is running). */
const isConfigured = (key: string): boolean => {
  if (settingsStore.config?.wallpaper_mode?.mode !== "fixed") return false;

  const configured = fixedStore.configThemes[key];
  if (configured === undefined) return false;
  // "all" is a pseudo-monitor: an individual map never stores it.
  if (!fixedStore.configUnified && key === "all") return false;

  return (fixedStore.monitorThemes[key] ?? configured) === configured;
};

/** Whether this scope is actually in effect right now: written into the config and the engine is running. */
const isApplied = (key: string): boolean =>
  engineStore.running && isConfigured(key);

/** Apply/stop that scope (write the config and restart the engine). */
const toggleApply = async (key: string) => {
  const config = settingsStore.config;
  if (!config) return;
  const theme = fixedStore.monitorThemes[key] ?? configuredThemes()[key];
  if (!theme) return;

  let next: MonitorThemes;
  try {
    next = await resolveMonitorThemes(config.wallpaper_mode, monitorIds(), {
      monitorId: key,
      themeId: theme,
      unified: fixedStore.allUnified,
      applied: isApplied(key),
    });
  } catch (e) {
    log.error("Failed to resolve the monitor theme change", e);
    return;
  }

  // Mirror applyWallpaperMode's optimistic config update, so the stage reflects the
  // new mapping while the daemon restarts.
  setFixedStore({
    configThemes: toRecord(next.themes),
    configUnified: next.unified,
  });

  try {
    await applyWallpaperMode(next.mode);
  } catch {
    // applyWallpaperMode has already reported the failure to the user; the callers are
    // fire-and-forget, so the rejection must not escape as an unhandled one.
  }
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
  isApplied,
  toggleApply,
  toggleUnified,
  selectMon,
  selectWallpaper,
  setMonitorTheme,
  syncFromConfig,
};
