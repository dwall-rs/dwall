import { createSignal } from "solid-js";

/**
 * Theme base state management Hook, used to manage theme-related base states
 * @returns Theme base states and related methods
 */
export const useThemeState = () => {
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [menuItemIndex, setMenuItemIndex] = createSignal<number | undefined>(0);
  const [themeExists, setThemeExists] = createSignal(false);
  const [showSettings, setShowSettings] = createSignal(false);

  return {
    appliedThemeID,
    setAppliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    setThemeExists,
    showSettings,
    setShowSettings,
  };
};
