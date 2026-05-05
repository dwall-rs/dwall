import { createSignal } from "solid-js";

/**
 * Theme base state management Hook, used to manage theme-related base states
 * @returns Theme base states and related methods
 */
export const useThemeState = () => {
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadingTheme, setDownloadingTheme] = createSignal(false);

  return {
    appliedThemeID,
    setAppliedThemeID,
    downloadingTheme,
    setDownloadingTheme,
  };
};
