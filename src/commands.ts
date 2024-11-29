import { invoke } from "@tauri-apps/api/core";

export const showWindow = async (label: string) =>
  invoke<void>("show_window", { label });

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const checkThemeExists = async (themeId: string) =>
  invoke<void>("check_theme_exists", { themeId });

export const closeLastThemeTask = async () =>
  invoke<void>("close_last_theme_task");

export const applyTheme = async (config: Config) =>
  invoke("apply_theme", { config });

export const getAppliedThemeID = async () =>
  invoke<string | null>("get_applied_theme_id");

export const checkAutoStart = async () => invoke<boolean>("check_auto_start");

export const enableAutoStart = async () => invoke<void>("enable_auto_start");

export const disableAutoStart = async () => invoke<void>("disable_auto_start");

export const downloadThemeAndExtract = async (
  config: Config,
  themeId: string,
) => invoke<void>("download_theme_and_extract", { config, themeId });
