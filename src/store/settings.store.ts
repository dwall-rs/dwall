// Responsibility: preferences — startup/coordinate/lock-screen toggles, check interval, mirror template,
//       language. Inline fields carry their own saved snapshot.
//       Note: "auto follow system" does not live here — it belongs to the theme domain (see theme.store).

import { createStore, produce } from "solid-js/store";
import type {
  Config,
  PositionSource,
  PositionSourceAutomatic,
  PositionSourceManual,
  WallpaperMode,
} from "@/domain/config";
import type { Socks5 } from "@/domain/config";
import { isSocks5 } from "@/domain/config";
import {
  applyTheme,
  moveDirectory,
  readConfigFile,
  writeConfigFile,
} from "@/ipc";
import { logger } from "@/utils";
import { refresh as refreshEngine } from "./engine.store";
import { setMode, themeStore } from "./theme.store";

const log = logger.child("settings");

type Status = "idle" | "loading" | "ready" | "error";

interface SettingsState {
  status: Status;
  error: string | null;
  saveError: string | null;
  config: Config | null;
  saved: Config | null;
  saving: boolean;
}

const [settingsStore, setSettingsStore] = createStore<SettingsState>({
  status: "idle",
  error: null,
  saveError: null,
  config: null,
  saved: null,
  saving: false,
});

const load = async () => {
  setSettingsStore("status", "loading");
  try {
    const cfg = await readConfigFile();
    setSettingsStore((prev) => ({
      ...prev,
      config: cfg,
      saved: cfg,
      status: "ready",
    }));
    // Seed the appearance from the persisted preference: auto_detect ⇄ follow the system
    if (cfg.auto_detect_color_scheme) setMode("system");
    else if (themeStore.mode === "system") setMode("light");
  } catch (e) {
    setSettingsStore((prev) => ({
      ...prev,
      status: "error",
      error: String(e),
    }));
  }
};

const patch = (p: Partial<Config>) => {
  setSettingsStore("config", (prev) => (prev ? { ...prev, ...p } : {}));
};

const setPositionType = (t: PositionSource["type"]) => {
  setSettingsStore("config", (prev) =>
    prev
      ? {
          ...prev,
          position_source:
            t === "AUTOMATIC"
              ? {
                  type: "AUTOMATIC",
                  update_on_each_calculation: false,
                  cache_minutes: 30,
                }
              : { type: "MANUAL", latitude: 0, longitude: 0, altitude: 875 },
        }
      : {},
  );
};

const patchPosition = (
  p: Partial<PositionSourceAutomatic | PositionSourceManual>,
) => {
  setSettingsStore("config", (prev) =>
    prev ? { ...prev, position_source: { ...prev.position_source, ...p } } : {},
  );
};

export type NetworkType = "none" | "mirror" | "socks5";

const networkType = (): NetworkType => {
  const network = settingsStore.config?.network;
  if (network == null) return "none";

  return typeof network === "string" ? "mirror" : "socks5";
};

const setNetworkType = (t: NetworkType) => {
  if (!settingsStore.config) return;

  setSettingsStore(
    "config",
    "network",
    t === "none"
      ? null
      : t === "mirror"
        ? ""
        : { host: "127.0.0.1", port: 1080 },
  );
};

const patchSocks5 = (p: Partial<Socks5>) => {
  setSettingsStore("config", (prev) =>
    prev && isSocks5(prev.network)
      ? { ...prev, network: { ...prev.network, ...p } }
      : {},
  );
};

const save = async () => {
  if (!settingsStore.config || settingsStore.saving) return;

  setSettingsStore(
    produce((settings) => {
      settings.saving = true;
      settings.saveError = null;
    }),
  );

  try {
    await writeConfigFile(settingsStore.config);
    setSettingsStore(
      produce((settings) => {
        settings.saved = { ...settings.config } as Config;
        settings.saving = false;
      }),
    );
  } catch (e) {
    setSettingsStore(
      produce((settings) => {
        settings.saving = false;
        settings.saveError = String(e);
      }),
    );
  }
};

/**
 * Write wallpaper_mode and apply it to the engine (write the config file + restart the daemon).
 * Both fixed and random mode "apply/save" go through here.
 */
const applyWallpaperMode = async (mode: WallpaperMode) => {
  const config = settingsStore.config;
  if (!config) return;

  const next: Config = { ...config, wallpaper_mode: mode };
  setSettingsStore("config", next);

  try {
    await applyTheme(next);
    // Apply/stop starts or stops the daemon, so sync the engine state right away.
    void refreshEngine();
    setSettingsStore(
      produce((s) => {
        s.saved = { ...next } as Config;
        s.saveError = null;
      }),
    );
  } catch (e) {
    setSettingsStore("saveError", String(e));
    throw e;
  }
};

const discard = () =>
  setSettingsStore((s) =>
    s.saved ? { config: { ...s.saved }, saveError: null } : {},
  );

/** Move the themes directory to `dir` (when it changed) and persist the config. */
const changeThemesDirectory = async (dir: string) => {
  try {
    const current = settingsStore.config?.themes_directory;
    if (current && current !== dir) {
      await moveDirectory(current, dir);
    }
    patch({ themes_directory: dir });
    await save();
  } catch (e) {
    log.error("Failed to move themes directory", e);
  }
};

/** Derived dirty state: deep-compare the whole config. */
const isConfigDirty = (s: SettingsState): boolean =>
  !!s.config &&
  !!s.saved &&
  JSON.stringify(s.config) !== JSON.stringify(s.saved);

export {
  settingsStore,
  load,
  patch,
  setPositionType,
  patchPosition,
  networkType,
  setNetworkType,
  patchSocks5,
  save,
  applyWallpaperMode,
  discard,
  isConfigDirty,
  changeThemesDirectory,
};
