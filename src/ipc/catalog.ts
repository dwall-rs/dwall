import { invoke } from "@tauri-apps/api/core";

import type { CatalogTheme, SolarAngle } from "./types";

/** Themes available for download */
export const getThemeCatalog = async () =>
  invoke<CatalogTheme[]>("get_theme_catalog");

/** Wallpapers (with target solar angles) of an installed theme; empty if not installed */
export const getThemeWallpapers = async (themeId: string) =>
  invoke<SolarAngle[]>("get_theme_wallpapers", { themeId });

/**
 * Position (in the theme's solar-angle list) of the entry closest to the given
 * solar position, or null. A theme may reuse one image for several positions,
 * so this identifies the exact matched entry, not just its image index.
 */
export const matchWallpaper = async (
  themeId: string,
  altitude: number,
  azimuth: number,
) => invoke<number | null>("match_wallpaper", { themeId, altitude, azimuth });
