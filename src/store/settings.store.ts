// Responsibility: preferences — startup/coordinate/lock-screen toggles, check interval, mirror template,
//       language. Inline fields carry their own saved snapshot.
//       Note: "auto follow system" does not live here — it belongs to the theme domain (see theme.store).

import { createStore, produce } from "solid-js/store";
import type {
  Config,
  PositionSource,
  PositionSourceManual,
  WallpaperMode,
} from "@/models/config";
import type { Socks5 } from "@/models/config";
import { isSocks5 } from "@/models/config";
import {
  applyTheme,
  moveDirectory,
  readConfigFile,
  writeConfigFile,
} from "@/ipc";
import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { logger } from "@/utils";
import { refresh as refreshEngine } from "./engine.store";

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

/**
 * A deep copy of a configuration, for the "saved" snapshot.
 *
 * A shallow copy is not enough: `setStore` merges into the existing object rather
 * than replacing it, so any object still shared with the draft is updated in place
 * by the next edit. `load` used to store the very same object as both the draft and
 * the snapshot, which made `isConfigDirty` permanently false on a freshly opened
 * window — the header's Save and Discard buttons never enabled.
 */
const snapshot = (config: Config): Config =>
  JSON.parse(JSON.stringify(config)) as Config;

/**
 * Reads the configuration into the draft.
 *
 * Also the retry path behind the settings view's error panel: a failed read used
 * to leave the page looking like an empty but working form, with `config` staying
 * `null` forever and nothing to click to try again.
 */
const load = async () => {
  setSettingsStore((prev) => ({
    ...prev,
    status: "loading",
    error: null,
    saveError: null,
  }));

  try {
    const cfg = await readConfigFile();
    setSettingsStore((prev) => ({
      ...prev,
      config: cfg,
      saved: snapshot(cfg),
      status: "ready",
      error: null,
    }));
  } catch (e) {
    setSettingsStore((prev) => ({
      ...prev,
      status: "error",
      error: String(e),
    }));
  }
};

/**
 * Merges a partial config into the draft.
 *
 * Does nothing while the configuration is missing: the sections used to be
 * rendered before the load finished, and a click then turned `config` into `{}`
 * (the path-scoped setter stores whatever the updater returns), which made every
 * later `config.position_source.type` read throw. Callers only ever run from a
 * rendered setting, and those now wait for the load, so this is the second guard.
 */
const patch = (p: Partial<Config>) => {
  if (!settingsStore.config) return;

  setSettingsStore("config", (prev) => (prev ? { ...prev, ...p } : prev));
};

const setPositionType = (t: PositionSource["type"]) => {
  if (!settingsStore.config) return;

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
      : prev,
  );
};

/**
 * Updates the manual coordinates.
 *
 * Only meaningful in manual mode, and only the manual fields are ever patched
 * here, so the patch is applied to that variant explicitly: spreading the
 * `PositionSource` union would widen it into a shape neither variant accepts (and
 * would slip coordinates into the automatic variant).
 */
const patchPosition = (p: Partial<PositionSourceManual>) => {
  const current = settingsStore.config?.position_source;
  if (current?.type !== "MANUAL") return;

  setSettingsStore("config", {
    position_source: { ...current, ...p },
  });
};

/**
 * Enables or disables the daemon's system light/dark switching.
 *
 * The daemon reads the flag from the config and reloads when the file changes, so the
 * next cycle applies it. It is deliberately independent of this window's own appearance:
 * the appearance control owns that, and the window can only follow the system if the user
 * picks "System" there.
 */
const setAutoDetectColorScheme = (enabled: boolean) => {
  patch({ auto_detect_color_scheme: enabled });
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
  if (!settingsStore.config) return;

  setSettingsStore("config", (prev) =>
    prev && isSocks5(prev.network)
      ? { ...prev, network: { ...prev.network, ...p } }
      : prev,
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

  // Snapshot before awaiting: an edit made while the write is in flight is not in
  // the file that was sent, so it must stay dirty after the save completes.
  const toSave = snapshot(settingsStore.config);

  try {
    await writeConfigFile(toSave);
    setSettingsStore(
      produce((settings) => {
        settings.saved = toSave;
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
    toast.error(t("common.operationFailed"), { description: String(e) });
  }
};

/**
 * Write wallpaper_mode and apply it to the engine (write the config file + restart the daemon).
 * Both fixed and random mode "apply/save" go through here.
 */
const applyWallpaperMode = async (mode: WallpaperMode) => {
  const draft = settingsStore.config;
  const saved = settingsStore.saved;
  if (!draft || !saved) return;

  // Apply only the mode. The draft may carry edits the user has not saved yet, and
  // applying a theme from the stage must not write them to disk or mark them saved —
  // the old code applied `{ ...draft, wallpaper_mode }`, which committed them silently.
  const applied: Config = { ...saved, wallpaper_mode: mode };

  // The mode itself is optimistic: the toggle reflects it before the daemon answers.
  setSettingsStore("config", { ...draft, wallpaper_mode: mode });

  try {
    await applyTheme(applied);
    // Apply/stop starts or stops the daemon, so sync the engine state right away.
    void refreshEngine();
    setSettingsStore(
      produce((s) => {
        s.saved = snapshot(applied);
        s.saveError = null;
      }),
    );
  } catch (e) {
    setSettingsStore("saveError", String(e));
    // Also surface it: the caller usually discards the rejection (`void apply…`).
    toast.error(t("common.operationFailed"), { description: String(e) });
    throw e;
  }
};

const discard = () =>
  setSettingsStore((s) =>
    s.saved ? { config: snapshot(s.saved), saveError: null } : {},
  );

/** Move the themes directory to `dir` (when it changed) and persist the config. */
const changeThemesDirectory = async (dir: string) => {
  const current = settingsStore.config?.themes_directory;
  if (!current || current === dir) return;

  try {
    await moveDirectory(current, dir);
  } catch (e) {
    // The move is what can fail here (a non-empty destination, a permission
    // problem). Without this the user's choice was discarded with only a log line,
    // while the row kept showing the old directory as if nothing had happened.
    log.error("Failed to move themes directory", e);
    toast.error(t("common.operationFailed"), { description: String(e) });
    return;
  }

  patch({ themes_directory: dir });
  await save();

  // `save` reports failure through the store instead of throwing. The directory is
  // already at the new path, so leaving the old config behind would make the next
  // start look at an empty directory; move it back and keep the old value.
  if (settingsStore.saveError) {
    patch({ themes_directory: current });
    log.error("Themes directory moved but the configuration was not saved");

    try {
      await moveDirectory(dir, current);
    } catch (e) {
      log.error("Failed to move the themes directory back", e);
      toast.error(t("common.operationFailed"), { description: String(e) });
    }
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
  setAutoDetectColorScheme,
  networkType,
  setNetworkType,
  patchSocks5,
  save,
  applyWallpaperMode,
  discard,
  isConfigDirty,
  changeThemesDirectory,
};
