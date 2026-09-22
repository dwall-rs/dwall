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

export const isSocks5 = (n: Network | undefined): n is Socks5 =>
  !!n && typeof n === "object" && "host" in n;

/**
 * 从配置里解析某个作用域当前使用的主题 id（仅固定模式）。
 * `All` 变体对所有作用域生效；`Individual` 按显示器 id 取。
 */
export const configuredThemeId = (
  config: Config | null | undefined,
  scopeKey: string,
): string | undefined => {
  const mode = config?.wallpaper_mode;
  if (mode?.mode !== "fixed") return undefined;
  const wallpapers = mode.monitor_specific_wallpapers;
  if (typeof wallpapers === "string") return wallpapers;
  // 「所有显示器」作用域下，取首个已配置的显示器主题作为代表。
  if (scopeKey === "all") return Object.values(wallpapers)[0];
  return wallpapers[scopeKey];
};
