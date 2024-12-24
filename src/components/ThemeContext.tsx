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
    window
      .matchMedia("(prefers-color-scheme: dark)")
      .addEventListener("change", (event) => {
        if (event.matches) {
          setTitlebarColorMode("DARK");
        } else {
          setTitlebarColorMode("LIGHT");
        }
      });
  });

  const onMenuItemClick = async (idx: number) => {
    setMenuItemIndex(idx);
    try {
      await checkThemeExists(config()!.themes_directory, themes[idx].id);
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
    }
  };

  const onCloseTask = async () => {
    const stoppedConfig = { ...config()!, selected_theme_id: undefined };
    applyTheme(stoppedConfig);
    refetchConfig();

    setAppliedThemeID();
  };

  const onApply = async () => {
    try {
      await requestLocationPermission();
    } catch (e) {
      const ok = await ask(
        translate(translations()!, "message-location-permission"),
        {
          kind: "warning",
        }
      );

      if (!ok) {
        exit(0);
      } else {
        mutate((prev) => ({
          ...prev!,
          coordinate_source: { type: "MANUAL" },
        }));
        setShowSettings(true);
      }
      return;
    }

    const theme = currentTheme();
    if (!theme) return;

    const newConfig = {
      ...config()!,
      selected_theme_id: theme.id,
    };
    await applyTheme(newConfig);
    refetchConfig();
    setAppliedThemeID(newConfig.selected_theme_id);
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
    onMenuItemClick,
    onCloseTask,
    onApply,
    update,
    recheckUpdate,
    showSettings,
    setShowSettings,
    translations,
  };
};
