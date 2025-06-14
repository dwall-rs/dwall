import { invoke } from "@tauri-apps/api/core";

export const validateTheme = async (themesDirecotry: string, themeId: string) =>
  invoke<void>("validate_theme", { themesDirecotry, themeId });

export const applyTheme = async (config: Config) =>
  invoke<void>("apply_theme", { config });

export const getAppliedThemeID = async (monitorId: string) =>
  invoke<string | null>("get_applied_theme_id", { monitorId });

export const downloadThemeAndExtract = async (
  config: Config,
  themeId: string,
) => invoke<void>("download_theme_and_extract", { config, themeId });

export const cancelThemeDownload = async (themeId: string) =>
  invoke<void>("cancel_theme_download", { themeId });

export const getOrSaveCachedThumbnails = async (
  themeId: string,
  serialNumber: number,
  url: string,
) =>
  invoke<string>("get_or_save_cached_thumbnails", {
    themeId,
    serialNumber,
    url,
  });

export const clearThumbnailCache = async () =>
  invoke<number>("clear_thumbnail_cache");

export const moveThemesDirectory = async (config: Config, dirPath: string) =>
  invoke<void>("move_themes_directory", { config, dirPath });
