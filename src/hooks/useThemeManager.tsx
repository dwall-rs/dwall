import { useThemeState } from "./theme/useThemeState";
import { useMonitorSelection } from "./monitor/useMonitorSelection";
import { useLocationPermission } from "./useLocationPermission";
import { useThemeApplication } from "./theme/useThemeApplication";
import { useThemeSelection } from "./theme/useThemeSelection";
import { useMonitorThemeSync } from "./monitor/useMonitorThemeSync";

/**
 * Theme management Hook, used to combine various sub-Hooks and provide a unified interface
 * @param themes List of available themes
 * @returns Theme management related states and methods
 */
export const useThemeManager = (themes: ThemeItem[]) => {
  // Use base state Hook
  const themeState = useThemeState();
  const {
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
  } = themeState;

  // Use monitor selection Hook
  const {
    monitorID,
    setMonitorID,
    monitors,
    monitorSpecificThemes,
    monitorSpecificThemesIsSame,
    handleMonitorChange,
  } = useMonitorSelection(config);

  // Use theme selection Hook
  const { currentTheme, handleThemeSelection } = useThemeSelection(
    themes,
    config,
    menuItemIndex,
    setMenuItemIndex,
    setThemeExists,
  );

  // Use location permission Hook
  const { checkLocationPermission } = useLocationPermission(
    mutate,
    setShowSettings,
  );

  // Use theme application Hook
  const { handleThemeApplication, handleTaskClosure } = useThemeApplication(
    config,
    refetchConfig,
    currentTheme,
    monitorID,
    monitorSpecificThemes,
    checkLocationPermission,
    setAppliedThemeID,
  );

  // Use Hook for monitoring display selection changes
  useMonitorThemeSync(
    themes,
    monitorID,
    config,
    monitorSpecificThemesIsSame,
    setAppliedThemeID,
    setMenuItemIndex,
  );

  // Return states and methods organized by functional areas
  return {
    // 主题相关
    theme: {
      currentTheme,
      appliedThemeID,
      setAppliedThemeID,
      downloadThemeID,
      setDownloadThemeID,
      menuItemIndex,
      setMenuItemIndex,
      themeExists,
      handleThemeSelection,
      handleThemeApplication,
    },

    // 配置相关
    config: {
      data: config,
      refetch: refetchConfig,
      mutate,
    },

    // 显示器相关
    monitor: {
      id: monitorID,
      setId: setMonitorID,
      list: monitors,
      specificThemes: monitorSpecificThemes,
      allSameTheme: monitorSpecificThemesIsSame,
      handleChange: handleMonitorChange,
    },

    // 任务相关
    task: {
      handleClosure: handleTaskClosure,
    },

    // 设置相关
    settings: {
      show: showSettings,
      setShow: setShowSettings,
    },

    // 更新相关
    update: {
      data: update,
      recheck: recheckUpdate,
    },
  };
};

// To maintain backward compatibility, export the original name
export const useThemeSelector = useThemeManager;
