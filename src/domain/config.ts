/* ===== src/domain/config.ts ===== */
// 职责：真实配置结构（与引擎约定一致）+ 默认值 + 类型守卫。
export type GithubMirrorTemplate = string;
export interface Socks5 {
  host: string;
  port: number;
}
export type Network = GithubMirrorTemplate | Socks5;

export interface FixedMode {
  mode: "fixed";
  monitor_specific_wallpapers: string | Record<string, string>;
}
export interface RandomMode {
  mode: "random";
  pool: string[] | null;
}
export type WallpaperMode = FixedMode | RandomMode;

export interface PositionSourceAutomatic {
  type: "AUTOMATIC";
  update_on_each_calculation?: boolean;
  cache_minutes?: number;
}
export interface PositionSourceManual {
  type: "MANUAL";
  latitude?: number;
  longitude?: number;
  altitude?: number;
}
export type PositionSource = PositionSourceAutomatic | PositionSourceManual;

export interface Config {
  network?: Network;
  selected_theme_id?: string;
  interval: number;
  image_format: string;
  themes_directory: string;
  customized_themes_directory: string;
  position_source: PositionSource;
  auto_detect_color_scheme: boolean;
  lock_screen_wallpaper_enabled: boolean;
  monitor_specific_wallpapers: string | Record<string, string>;
  title_bar_color_follows_windows_theme: boolean;
  wallpaper_mode?: WallpaperMode;
}

export const isSocks5 = (n: Network | undefined): n is Socks5 =>
  !!n && typeof n === "object" && "host" in n;
