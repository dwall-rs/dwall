import { invoke } from "@tauri-apps/api/core";

export const showMainWindow = async () => invoke<void>("show_main_window");

export const readConfigFile = async () => invoke<Config>("read_config_file");

export const checkThemeExists = async (id: string) =>
  invoke<void>("check_theme_exists", { id });

export const closeLastThemeTask = async () =>
  invoke<void>("close_last_theme_task");
