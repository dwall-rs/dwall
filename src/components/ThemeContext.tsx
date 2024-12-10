import { createSignal, createMemo, createResource } from "solid-js";
import { check } from "@tauri-apps/plugin-updater";
import {
  applyTheme,
  checkThemeExists,
  closeLastThemeTask,
  readConfigFile,
  requestLocationPermission,
} from "~/commands";

export const useThemeSelector = (themes: ThemeItem[]) => {
  const [config, { refetch: refetchConfig }] =
    createResource<Config>(readConfigFile);
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [index, setIndex] = createSignal(0);
  const [themeExists, setThemeExists] = createSignal(false);
  const [update, { refetch: recheckUpdate }] = createResource(() => check());

  const currentTheme = createMemo(() => themes[index()]);

  // const autoRun = async (config: Config) => {
  //   const { selected_theme_id, ...themeParams } = config;
  //   if (!selected_theme_id) return;

  //   await applyTheme({
  //     selected_theme_id,
  //     ...themeParams,
  //   });

  //   setAppliedThemeID(selected_theme_id);
  //   setIndex(themes.findIndex((t) => t.id === selected_theme_id));
  // };

  const onMenuItemClick = async (idx: number) => {
    setIndex(idx);
    try {
      await checkThemeExists(themes[idx].id);
      setThemeExists(true);
    } catch (e) {
      setThemeExists(false);
    }
  };

  const onCloseTask = async () => {
    closeLastThemeTask();

    const stoppedConfig = { ...config()!, selected_theme_id: undefined };
    applyTheme(stoppedConfig);
    refetchConfig();

    setAppliedThemeID();
  };

  const onApply = async () => {
    try {
      await requestLocationPermission();
    } catch (e) {
      // TODO: handle error
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
