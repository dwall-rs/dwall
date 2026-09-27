// Frontend-facing config types. The Rust `Config` is the single source of truth; these
// are re-exported from the ts-rs generated bindings (see src/ipc/bindings).
export type {
  Config,
  ImageFormat,
  MonitorSpecificWallpapers,
  Network,
  PositionSource,
  WallpaperMode,
} from "@/ipc/types";

import type { Config, Network, PositionSource } from "@/ipc/types";

export type PositionSourceAutomatic = Extract<
  PositionSource,
  { type: "AUTOMATIC" }
>;
export type PositionSourceManual = Extract<PositionSource, { type: "MANUAL" }>;

export interface Socks5 {
  host: string;
  port: number;
}

export const isSocks5 = (n: Network | null | undefined): n is Socks5 =>
  !!n && typeof n === "object" && "host" in n;

/**
 * Resolve the theme id currently used by a scope from the config (fixed mode only).
 * The `All` variant applies to every scope; `Individual` is looked up by monitor id.
 */
export const configuredThemeId = (
  config: Config | null | undefined,
  scopeKey: string,
): string | undefined => {
  const mode = config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return undefined;
  const wallpapers = mode.monitor_specific_wallpapers;
  if (typeof wallpapers === "string") return wallpapers;
  // For the "all monitors" scope, take the first configured monitor's theme as the representative.
  if (scopeKey === "all") return Object.values(wallpapers)[0];
  return wallpapers[scopeKey];
};
