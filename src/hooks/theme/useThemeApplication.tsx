import { applyTheme } from "~/commands";

/**
 * Theme application management Hook, used to handle theme application and task closure
 * @param config Application configuration
 * @param refetchConfig Function to refetch configuration
 * @param currentTheme Currently selected theme
 * @param monitorID Currently selected monitor ID
 * @param monitorSpecificThemes Monitor-specific theme configuration
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
  // Handle theme application
  const handleThemeApplication = async (
    monitorID: () => string,
    monitorSpecificThemes: () => [string, string][],
  ) => {
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
    const monitorSpecificWallpapers: Record<string, string> = {
      ...currentConfig.monitor_specific_wallpapers,
    };

    if (monitorID() !== "all") {
      // Set theme for specific monitor
      monitorSpecificWallpapers[monitorID()!] = theme.id;
    } else {
      // Set the same theme for all monitors
      for (const [id, _] of monitorSpecificThemes()) {
        monitorSpecificWallpapers[id] = theme.id;
      }
    }

    currentConfig.monitor_specific_wallpapers = monitorSpecificWallpapers;

    if (!currentConfig.selected_theme_id) {
      currentConfig.selected_theme_id = theme.id;
    }

    try {
      await applyTheme(currentConfig);
      refetchConfig();
      setAppliedThemeID(theme.id);
    } catch (e) {
      console.error("Failed to apply theme:", e);
    }
  };

  return {
    handleThemeApplication,
  };
};
