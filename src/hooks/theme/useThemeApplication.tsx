import { message } from "@tauri-apps/plugin-dialog";
import { applyTheme } from "~/commands";
import { t } from "~/i18n";

/**
 * Theme application management Hook, used to handle theme application and task closure
 * @param config Application configuration
 * @param refetchConfig Function to refetch configuration
 * @param currentTheme Currently selected theme
 * @param checkLocationPermission Function to check location permission
 * @param setAppliedThemeID Function to set the applied theme ID
 * @returns Theme application related methods
 */
export const useThemeApplication = (
  config: () => Config | undefined,
  refetchConfig: () => void,
  checkLocationPermission: () => Promise<boolean>,
  setAppliedThemeID: (id?: string) => void,
) => {
  // Handle theme application
  const handleThemeApplication = async (
    monitorID: Accessor<string>,
    themeID?: string,
  ) => {
    if (!themeID || !config()) return;

    const currentConfig = config()!;

    // Check coordinate configuration
    const hasValidCoordinates =
      currentConfig.position_source?.type === "MANUAL" &&
      typeof currentConfig.position_source.latitude === "number" &&
      typeof currentConfig.position_source.longitude === "number";

    if (!hasValidCoordinates) {
      const hasPermission = await checkLocationPermission();
      if (!hasPermission) return;
    }

    // Update monitor-specific wallpaper configuration
    let monitorSpecificWallpapers: Record<string, string> | string;
    // const monitorSpecificWallpapers: Record<string, string> = {
    //   ...currentConfig.monitor_specific_wallpapers,
    // };

    if (monitorID() === "all") {
      // Set the same theme for all monitors
      monitorSpecificWallpapers = themeID;
    } else {
      // Set theme for specific monitor
      monitorSpecificWallpapers =
        typeof currentConfig.monitor_specific_wallpapers === "string"
          ? {}
          : { ...currentConfig.monitor_specific_wallpapers };

      monitorSpecificWallpapers[monitorID()!] = themeID;
    }

    currentConfig.monitor_specific_wallpapers = monitorSpecificWallpapers;

    try {
      await applyTheme(currentConfig);
      refetchConfig();
      setAppliedThemeID(themeID);
    } catch (e) {
      message(t("theme.message.applyThemeFailed", { error: String(e) }), {
        kind: "error",
      });
    }
  };

  return {
    handleThemeApplication,
  };
};
