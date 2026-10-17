import { invoke } from "@tauri-apps/api/core";

import type { ColorScheme } from "./types";

export const showWindow = async (label: string) =>
  invoke<void>("show_window", { label });

export const openUrl = async (url: string) => invoke<void>("open_url", { url });

/**
 * Paints the window's title bar for `colorMode`.
 *
 * `followWindowsTheme` hands the caption back to Windows instead of matching Dwall's
 * background; the backend decides what the running Windows version supports (see
 * `crates/dwall-settings/src/window.rs`).
 */
export const setTitlebarColorMode = async (
  colorMode: ColorScheme,
  followWindowsTheme: boolean,
) =>
  invoke<void>("set_titlebar_color_mode", {
    colorMode,
    followWindowsTheme,
  });
