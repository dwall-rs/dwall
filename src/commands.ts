import { invoke } from "@tauri-apps/api/core";

export const showMainWindow = async () => invoke<void>("show_main_window");
