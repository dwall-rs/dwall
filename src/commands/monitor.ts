import { invoke } from "@tauri-apps/api/core";

export const getMonitors = async () =>
  invoke<Record<string, MonitorInfo>>("get_monitors");
