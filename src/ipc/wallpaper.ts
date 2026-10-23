import { invoke } from "@tauri-apps/api/core";

import type { MonitorThemeChange, MonitorThemes, WallpaperMode } from "./types";

/**
 * Resolve the fixed mode's wallpapers into the effective theme per scope.
 *
 * With a `change`, the result also carries the wallpaper mode that apply/stop writes,
 * so the renderer never builds the monitor union itself.
 */
export const resolveMonitorThemes = async (
  mode: WallpaperMode,
  monitors: string[],
  change: MonitorThemeChange | null = null,
) =>
  invoke<MonitorThemes>("resolve_monitor_themes_cmd", {
    mode,
    monitors,
    change,
  });
