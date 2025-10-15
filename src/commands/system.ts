import { invoke } from "@tauri-apps/api/core";

export const requestLocationPermission = async () =>
  invoke<void>("request_location_permission");

export const openDir = async (dirPath: string) =>
  invoke<void>("open_dir", { dirPath });

export const killDaemon = async () => invoke<void>("kill_daemon_cmd");
