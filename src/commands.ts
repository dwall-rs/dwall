import { invoke } from "@tauri-apps/api/core";

export const showMainWindow = async () => invoke<void>("show_main_window");

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const checkThemeExists = async (themeId: string) =>
  invoke<void>("check_theme_exists", { themeId });

export const closeLastThemeTask = async () =>
  invoke<void>("close_last_theme_task");

export const applyTheme = async (config: Config) =>
  invoke("apply_theme", { config });
