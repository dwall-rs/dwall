import { invoke } from "@tauri-apps/api/core";

import type { ColorScheme } from "./types";

export const showWindow = async (label: string) =>
  invoke<void>("show_window", { label });

export const openUrl = async (url: string) => invoke<void>("open_url", { url });

export const setTitlebarColorMode = async (colorMode: ColorScheme) =>
  invoke<void>("set_titlebar_color_mode", {
    colorMode,
  });
