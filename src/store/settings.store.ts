// 职责：偏好——开机/坐标/锁屏开关、检查间隔、镜像模板、语言。行内字段自带 saved 快照。
//       注意：「自动跟随系统」不在此处，它属于 theme 域（见 theme.store）。

import { createStore, produce } from "solid-js/store";
import type {
  Config,
  PositionSource,
  PositionSourceAutomatic,
  PositionSourceManual,
  Socks5,
} from "@/domain/config";
import { isSocks5 } from "@/domain/config";
import { readConfigFile, writeConfigFile } from "~/commands";
import { themeStore, setMode } from "./theme.store";

type Status = "idle" | "loading" | "ready" | "error";

interface SettingsState {
  status: Status;
  error: string | null;
  saveError: string | null;
  config: Config | null;
  saved: Config | null;
  saving: boolean;
  lang: string;
}

const [settingsStore, setSettingsStore] = createStore<SettingsState>({
  status: "idle",
  error: null,
  saveError: null,
  config: null,
  saved: null,
  saving: false,
  lang: "English",
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
    // 用持久化偏好初始化外观：auto_detect ⇄ 跟随系统
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

const setsetPositionType = (t: PositionSource["type"]) => {
  setSettingsStore("config", (prev) =>
    prev ? { ...prev, position_source: { type: t } } : {},
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
  if (network === undefined) return "none";

  return typeof network === "string" ? "mirror" : "socks5";
};

// FIXME: 修改网络不生效
const setNetworkType = (t: NetworkType) => {
  const cfg = settingsStore.config;
  if (!cfg) return;

  setSettingsStore(
    "config",
    "network",
    t === "none"
      ? undefined
      : t === "mirror"
        ? ""
        : { host: "127.0.0.1", port: 1080 },
  );
};

const patchSocks = (p: Partial<Socks5>) => {
  setSettingsStore("config", (prev) =>
    prev
      ? {
          ...prev,
          network_type: isSocks5(prev.network)
            ? { ...prev.network, ...p }
            : prev.network,
        }
      : {},
  );
};

const setWallpaperModeType = (m: "fixed" | "random") => {
  setSettingsStore("config", (prev) =>
    prev
      ? {
          ...prev,
          wallpaper_mode:
            m === "fixed"
              ? {
                  mode: "fixed",
                  monitor_specific_wallpapers: prev.monitor_specific_wallpapers,
                }
              : { mode: "random", pool: null },
        }
      : {},
  );
};

const setLang = (lang: string) => setSettingsStore("lang", lang);

// const setInterval = (interval: number) =>
//   setSettingsStore("interval", interval);

// const setMirror = (mirror: string) => setSettingsStore("mirror", mirror);

const save = async () => {
  if (!settingsStore.config || !settingsStore.saving) return;

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

const discard = () =>
  setSettingsStore((s) =>
    s.saved ? { config: { ...s.saved }, saveError: null } : {},
  );

/** 派生脏态：整份配置深比较。 */
const isConfigDirty = (s: SettingsState): boolean =>
  !!s.config &&
  !!s.saved &&
  JSON.stringify(s.config) !== JSON.stringify(s.saved);

export {
  settingsStore,
  load,
  patch,
  setsetPositionType,
  patchPosition,
  networkType,
  setNetworkType,
  patchSocks,
  setWallpaperModeType,
  setLang,
  save,
  discard,
  isConfigDirty,
};
