import { createSignal, createMemo, createResource, onMount } from "solid-js";
import { check } from "@tauri-apps/plugin-updater";
import {
  applyTheme,
  checkThemeExists,
  readConfigFile,
  requestLocationPermission,
  setTitlebarColorMode,
} from "~/commands";
import { message } from "@tauri-apps/plugin-dialog";

export const useThemeSelector = (themes: ThemeItem[]) => {
  const [config, { refetch: refetchConfig }] =
    createResource<Config>(readConfigFile);
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [index, setIndex] = createSignal(0);
  const [themeExists, setThemeExists] = createSignal(false);
  const [update, { refetch: recheckUpdate }] = createResource(() => check());

  const currentTheme = createMemo(() => themes[index()]);

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
    setIndex(idx);
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
      message("定位权限未打开，请手动开启定位", { kind: "error" });
      return;
    }

    const newConfig = {
      ...config()!,
      selected_theme_id: currentTheme().id,
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
    index,
    setIndex,
    themeExists,
    currentTheme,
    onMenuItemClick,
    onCloseTask,
    onApply,
    update,
    recheckUpdate,
  };
};
