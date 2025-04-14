import { invoke } from "@tauri-apps/api/core";

export const showWindow = async (label: string) =>
  invoke<void>("show_window", { label });

export const setTitlebarColorMode = async (colorMode: ColorMode) =>
  invoke<void>("set_titlebar_color_mode", { colorMode });
