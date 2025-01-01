import { createSignal, createMemo, createResource, onMount } from "solid-js";
import { check } from "@tauri-apps/plugin-updater";
import {
  applyTheme,
  checkThemeExists,
  getTranslations,
  readConfigFile,
  requestLocationPermission,
  setTitlebarColorMode,
} from "~/commands";
import { ask } from "@tauri-apps/plugin-dialog";
import { exit } from "@tauri-apps/plugin-process";
import { translate } from "~/utils/i18n";

export const useThemeSelector = (themes: ThemeItem[]) => {
  const [translations] = createResource(getTranslations);
  const [config, { refetch: refetchConfig, mutate }] =
    createResource<Config>(readConfigFile);
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [menuItemIndex, setMenuItemIndex] = createSignal<number | undefined>(0);
  const [themeExists, setThemeExists] = createSignal(false);
  const [update, { refetch: recheckUpdate }] = createResource(() => check());
  const [showSettings, setShowSettings] = createSignal(false);

  const currentTheme = createMemo(() => {
    const idx = menuItemIndex();
    if (idx === undefined) return;
    return themes[idx];
  });

  onMount(() => {
    const darkModeMediaQuery = window.matchMedia(
      "(prefers-color-scheme: dark)",
    );

    const handleColorSchemeChange = (event: MediaQueryListEvent) => {
      setTitlebarColorMode(event.matches ? "DARK" : "LIGHT");
    };

    darkModeMediaQuery.addEventListener("change", handleColorSchemeChange);
    return () =>
      darkModeMediaQuery.removeEventListener("change", handleColorSchemeChange);
  });

  const handleThemeSelection = async (idx: number) => {
    setMenuItemIndex(idx);
    try {
      await checkThemeExists(config()?.themes_directory ?? "", themes[idx].id);
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
      console.error("Failed to check theme existence:", e);
    }
  };

  const handleTaskClosure = async () => {
    if (!config()) return;

    const updatedConfig = {
      ...config()!,
      selected_theme_id: undefined,
    };

    try {
      await applyTheme(updatedConfig);
      await refetchConfig();
      setAppliedThemeID(undefined);
    } catch (e) {
      console.error("Failed to close task:", e);
    }
  };

  const checkLocationPermission = async (): Promise<boolean> => {
    try {
      await requestLocationPermission();
      return true;
    } catch (e) {
      const shouldContinue = await ask(
        translate(translations()!, "message-location-permission"),
        { kind: "warning" },
      );

      if (!shouldContinue) {
        exit(0);
        return false;
      }

      mutate((prev) => ({
        ...prev!,
        coordinate_source: { type: "MANUAL" },
      }));
      setShowSettings(true);
      return false;
    }
  };

  const handleThemeApplication = async () => {
    const theme = currentTheme();
    if (!theme || !config()) return;

    const currentConfig = config()!;
    const hasValidCoordinates =
      currentConfig.coordinate_source?.type === "MANUAL" &&
      typeof currentConfig.coordinate_source.latitude === "number" &&
      typeof currentConfig.coordinate_source.longitude === "number";

    if (!hasValidCoordinates) {
      const hasPermission = await checkLocationPermission();
      if (!hasPermission) return;
    }

    try {
      const newConfig = {
        ...currentConfig,
        selected_theme_id: theme.id,
      };

      await applyTheme(newConfig);
      await refetchConfig();
      setAppliedThemeID(theme.id);
    } catch (e) {
      console.error("Failed to apply theme:", e);
    }
  };

  return {
    config,
    refetchConfig,
    appliedThemeID,
    setAppliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    menuItemIndex,
    setMenuItemIndex,
    themeExists,
    currentTheme,
    handleThemeSelection,
    handleTaskClosure,
    handleThemeApplication,
    update,
    recheckUpdate,
    showSettings,
    setShowSettings,
    translations,
  };
};
