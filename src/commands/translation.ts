import { invoke } from "@tauri-apps/api/core";

export const getTranslations = async () =>
  invoke<Record<TranslationKey, string>>("get_translations");
