import { createEffect } from "solid-js";
import { useConfig, useTheme } from "~/contexts";
import { navigateToTheme } from "~/router";
import { type ThemeID, themes } from "~/themes";

/**
 * Hook for monitoring display selection changes and synchronizing theme state
 * @param monitorID Currently selected monitor ID
 * @param monitorSpecificThemesIsSame Whether all monitors use the same theme
 * @returns No return value, only provides side effects
 */
export const useMonitorThemeSync = (
  monitorID: () => string,
  monitorSpecificThemesIsSame: () => boolean,
) => {
  const { data: config } = useConfig();
  const { setAppliedThemeID } = useTheme();

  // Monitor display selection changes, update theme state
  createEffect(async () => {
    const id = getThemeID(monitorID(), config()?.monitor_specific_wallpapers);

    if (!id || (!monitorSpecificThemesIsSame() && monitorID() === "all")) {
      setAppliedThemeID(undefined);
      navigateToTheme(themes[0].id);
    } else {
      setAppliedThemeID(id);
      navigateToTheme(id as ThemeID);
    }
  });
};

const getThemeID = (
  monitorID: string,
  monitor_specific_wallpapers?: Config["monitor_specific_wallpapers"],
) => {
  if (typeof monitor_specific_wallpapers === "string")
    return monitor_specific_wallpapers;

  return monitor_specific_wallpapers?.[monitorID];
};
