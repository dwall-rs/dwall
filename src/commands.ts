import { invoke } from "@tauri-apps/api/core";

export const showWindow = async (label: string) =>
  invoke<void>("show_window", { label });

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const writeConfigFile = async (config: Config) =>
  invoke<Config>("write_config_file", { config });

export const checkThemeExists = async (
  themesDirecotry: string,
  themeId: string,
) => invoke<void>("check_theme_exists", { themesDirecotry, themeId });

export const applyTheme = async (config: Config) =>
  invoke("apply_theme", { config });

export const getAppliedThemeID = async (monitorId: string) =>
  invoke<string | null>("get_applied_theme_id", { monitorId });

export const checkAutoStart = async () => invoke<boolean>("check_auto_start");

export const enableAutoStart = async () => invoke<void>("enable_auto_start");

export const disableAutoStart = async () => invoke<void>("disable_auto_start");

export const downloadThemeAndExtract = async (
  config: Config,
  themeId: string,
) => invoke<void>("download_theme_and_extract", { config, themeId });

export const cancelThemeDownload = async (themeId: string) =>
  invoke<void>("cancel_theme_download", { themeId });

export const requestLocationPermission = async () =>
  invoke<void>("request_location_permission");

export const setTitlebarColorMode = async (colorMode: ColorMode) =>
  invoke<void>("set_titlebar_color_mode", { colorMode });

export const openDir = async (dirPath: string) =>
  invoke<void>("open_dir", { dirPath });

export const moveThemesDirectory = async (config: Config, dirPath: string) =>
  invoke<void>("move_themes_directory", { config, dirPath });

export const openConfigDir = async () => invoke<void>("open_config_dir");

export const killDaemon = async () => invoke<void>("kill_daemon");

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

export const getTranslations = async () =>
  invoke<Record<TranslationKey, string>>("get_translations");

export const getMonitors = async () =>
  invoke<Record<string, MonitorInfo>>("get_monitors");
