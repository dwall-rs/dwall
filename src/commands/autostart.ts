import { invoke } from "@tauri-apps/api/core";

export const checkAutoStart = async () => invoke<boolean>("check_auto_start");

export const enableAutoStart = async () => invoke<void>("enable_auto_start");

export const disableAutoStart = async () => invoke<void>("disable_auto_start");
