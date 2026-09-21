import { invoke } from "@tauri-apps/api/core";

import type { CatalogTheme, SolarAngle } from "./types";

/** Themes available for download */
export const getThemeCatalog = async () =>
  invoke<CatalogTheme[]>("get_theme_catalog");

/** Wallpapers (with target solar angles) of an installed theme; empty if not installed */
export const getThemeWallpapers = async (themeId: string) =>
  invoke<SolarAngle[]>("get_theme_wallpapers", { themeId });

/** On-disk path of a theme wallpaper image, or null when missing */
export const getThemeWallpaperPath = async (themeId: string, index: number) =>
  invoke<string | null>("get_theme_wallpaper_path", { themeId, index });

/** Index of the wallpaper matching the given solar position, or null */
export const matchWallpaper = async (
  themeId: string,
  altitude: number,
  azimuth: number,
) => invoke<number | null>("match_wallpaper", { themeId, altitude, azimuth });
