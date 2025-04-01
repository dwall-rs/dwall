import { createEffect } from "solid-js";
import { useConfig, useTheme } from "~/contexts";
import { themes } from "~/themes";

/**
 * Hook for monitoring display selection changes and synchronizing theme state
 * @param themes List of available themes
 * @param monitorID Currently selected monitor ID
 * @param config Application configuration
 * @param monitorSpecificThemesIsSame Whether all monitors use the same theme
 * @param setAppliedThemeID Function to set the applied theme ID
 * @param setMenuItemIndex Function to set the current selected theme index
 * @returns No return value, only provides side effects
 */
export const useMonitorThemeSync = (
  monitorID: () => string,
  monitorSpecificThemesIsSame: () => boolean,
) => {
  const { data: config } = useConfig();
  const { setAppliedThemeID, setMenuItemIndex } = useTheme();

  // Monitor display selection changes, update theme state
  createEffect(() => {
    const id =
      monitorID() === "all"
        ? Object.values(config()?.monitor_specific_wallpapers ?? {})[0]
        : config()?.monitor_specific_wallpapers[monitorID()!];
    if (!id) return;

    if (!monitorSpecificThemesIsSame() && monitorID() === "all") {
      setAppliedThemeID(undefined);
      setMenuItemIndex(0);
    } else {
      const index = themes.findIndex((t) => t.id === id);
      setAppliedThemeID(id);
      setMenuItemIndex(index);
    }
  });
};
