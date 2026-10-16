import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

import type { Config, CustomizedTheme, ImageFormat } from "./types";

/** Progress of a running theme download, as reported by the `download-progress` event. */
export interface ThemeDownloadProgress {
  /** Identifies the download; the theme id. */
  taskId: string;
  downloadedBytes: number;
  totalBytes: number;
}

/**
 * Subscribes to theme download progress.
 *
 * Resolves to the unlisten function, so a caller that shows one download's
 * progress can stop listening when it ends.
 */
export const onThemeDownloadProgress = async (
  handler: (progress: ThemeDownloadProgress) => void,
) =>
  listen<{ task_id: string; downloaded_bytes: number; total_bytes: number }>(
    "download-progress",
    (event) =>
      handler({
        taskId: event.payload.task_id,
        downloadedBytes: event.payload.downloaded_bytes,
        totalBytes: event.payload.total_bytes,
      }),
  );

export const validateTheme = async (
  themesDirectory: string,
  themeId: string,
  imageFormat: ImageFormat,
  isCustomized = false,
) =>
  invoke<void>("validate_theme_cmd", {
    themesDirectory,
    themeId,
    isCustomized,
    imageFormat,
  });

export const applyTheme = async (config: Config) =>
  invoke<void>("apply_theme_cmd", { config });

export const getAppliedThemeID = async (monitorId: string) =>
  invoke<string | null>("get_applied_theme_id_cmd", { monitorId });

export const downloadThemeAndExtract = async (
  config: Config,
  themeId: string,
) => invoke<void>("download_theme_cmd", { config, themeId });

export const cancelThemeDownload = async (themeId: string) =>
  invoke<void>("cancel_theme_download_cmd", { themeId });

export const getOrSaveCachedThumbnails = async (
  themeId: string,
  serialNumber: number,
  url: string,
) =>
  invoke<string>("get_or_save_cached_thumbnails_cmd", {
    themeId,
    serialNumber,
    url,
  });

export const clearThumbnailCache = async () =>
  invoke<number>("clear_thumbnail_cache_cmd");

export const getCustomizedThemes = async (customizedThemesDirectory: string) =>
  invoke<CustomizedTheme[]>("get_customized_themes_cmd", {
    customizedThemesDirectory,
  });
