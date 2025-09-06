import { invoke } from "@tauri-apps/api/core";

export const getTranslations = async () =>
  invoke<Translations>("get_translations");
