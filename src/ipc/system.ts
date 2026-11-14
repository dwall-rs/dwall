import { invoke } from "@tauri-apps/api/core";

/** Windows user locale as a BCP-47 tag, for the default interface language. */
export const getSystemLocale = async () =>
  invoke<string | null>("get_system_locale");

export const requestLocationPermission = async () =>
  invoke<void>("request_location_permission");

export const openDir = async (dirPath: string) =>
  invoke<void>("open_dir", { dirPath });

export const openLogDir = async () => invoke<void>("open_log_dir");
