import { invoke } from "@tauri-apps/api/core";

export const openPrivacyLocationSettings = () =>
  invoke<void>("open_privacy_location_settings");
