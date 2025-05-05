import { message } from "@tauri-apps/plugin-dialog";
import { applyTheme } from "~/commands";
import { useTranslations } from "~/contexts";

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
  currentTheme: () => ThemeItem | undefined,
  checkLocationPermission: () => Promise<boolean>,
  setAppliedThemeID: (id?: string) => void,
) => {
  const { translateErrorMessage } = useTranslations();

  // Handle theme application
  const handleThemeApplication = async (monitorID: Accessor<string>) => {
    const theme = currentTheme();
    if (!theme || !config()) return;

    const currentConfig = config()!;

    // Check coordinate configuration
    const hasValidCoordinates =
      currentConfig.coordinate_source?.type === "MANUAL" &&
      typeof currentConfig.coordinate_source.latitude === "number" &&
      typeof currentConfig.coordinate_source.longitude === "number";

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
      monitorSpecificWallpapers = theme.id;
    } else {
      // Set theme for specific monitor
      monitorSpecificWallpapers =
        typeof currentConfig.monitor_specific_wallpapers === "string"
          ? {}
          : { ...currentConfig.monitor_specific_wallpapers };

      monitorSpecificWallpapers[monitorID()!] = theme.id;
    }

    currentConfig.monitor_specific_wallpapers = monitorSpecificWallpapers;

    try {
      await applyTheme(currentConfig);
      refetchConfig();
      setAppliedThemeID(theme.id);
    } catch (e) {
      message(translateErrorMessage("message-apply-theme-failed", e), {
        kind: "error",
      });
    }
  };

  return {
    handleThemeApplication,
  };
};
