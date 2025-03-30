import { createSignal, createResource } from "solid-js";
import { check } from "@tauri-apps/plugin-updater";
import { message } from "@tauri-apps/plugin-dialog";
import { readConfigFile } from "~/commands";
import { useTranslations } from "~/contexts";

/**
 * Theme base state management Hook, used to manage theme-related base states
 * @returns Theme base states and related methods
 */
export const useThemeState = () => {
  const { translateErrorMessage } = useTranslations();
  const [config, { refetch: refetchConfig, mutate }] =
    createResource<Config>(readConfigFile);
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [menuItemIndex, setMenuItemIndex] = createSignal<number | undefined>(0);
  const [themeExists, setThemeExists] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);

  // Update check
  const [update, { refetch: recheckUpdate }] = createResource(async () => {
    try {
      return await check();
    } catch (e) {
      console.log(e);
      message(translateErrorMessage("message-update-failed", e), {
        kind: "error",
      });
      return null;
    }
  });

  return {
    // States
    config,
    refetchConfig,
    mutate,
    appliedThemeID,
    setAppliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    setThemeExists,
    update,
    recheckUpdate,
    showSettings,
    setShowSettings,
  };
};
