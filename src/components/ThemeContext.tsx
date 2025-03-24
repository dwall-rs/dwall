import {
  createSignal,
  createMemo,
  createResource,
  onMount,
  createEffect,
} from "solid-js";
import { check } from "@tauri-apps/plugin-updater";
import {
  applyTheme,
  checkThemeExists,
  getMonitors,
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
  const [monitorInfoObject] = createResource(getMonitors);
  const [monitorID, setMonitorID] = createSignal<string>("all");

  const currentTheme = createMemo(() => {
    const idx = menuItemIndex();
    if (idx === undefined) return;
    return themes[idx];
  });

  const monitors = createMemo(() => {
    const monitorIDs = Object.keys(monitorInfoObject() ?? {}).sort();

    return [
      { value: "all", label: "全部" },
      ...monitorIDs.map((id) => ({
        value: monitorInfoObject()?.[id].device_path || "",
        label: monitorInfoObject()?.[id].friendly_name || "",
      })),
    ];
  });

  const monitorSpecificThemes = createMemo(() => {
    return Object.entries(config()?.monitor_specific_wallpapers ?? {}).sort(
      (a, b) => a[0].toLocaleLowerCase().localeCompare(b[0]),
    );
  });

  const monitorSpecificThemesIsSame = createMemo(() =>
    monitorSpecificThemes().every(
      (value) => value[1] === monitorSpecificThemes()[0][1],
    ),
  );

  createEffect(() => {
    if (!config()?.monitor_specific_wallpapers) return;

    const selectValue = monitorSpecificThemesIsSame()
      ? "all"
      : monitorSpecificThemes()[0][0];
    setMonitorID(selectValue);
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

  createEffect(() => {
    const id =
      monitorID() === "all"
        ? Object.values(config()?.monitor_specific_wallpapers ?? {})[0]
        : config()?.monitor_specific_wallpapers[monitorID()!];
    if (!id) return;

    if (!monitorSpecificThemesIsSame() && monitorID() === "all") {
      setAppliedThemeID();
      setMenuItemIndex(0);
    } else {
      const index = themes.findIndex((t) => t.id === id);
      setAppliedThemeID(id);
      setMenuItemIndex(index);
    }
  });

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

  const handleMonitorChange = (value: string) => {
    setMonitorID(value);
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

    const monitorSpecificWallpapers: Record<string, string> = {
      ...currentConfig.monitor_specific_wallpapers,
    };

    if (monitorID() !== "all") {
      monitorSpecificWallpapers[monitorID()!] = theme.id;
    } else {
      for (const [id, _] of monitorSpecificThemes()) {
        monitorSpecificWallpapers[id] = theme.id;
      }
    }

    currentConfig.monitor_specific_wallpapers = monitorSpecificWallpapers;

    if (!currentConfig.selected_theme_id) {
      currentConfig.selected_theme_id = theme.id;
    }

    try {
      const newConfig = {
        ...currentConfig,
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
    monitors,
    monitorID,
    handleMonitorChange,
  };
};
