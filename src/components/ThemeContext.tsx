import { createSignal, createMemo } from "solid-js";
import { applyTheme, checkThemeExists, closeLastThemeTask } from "~/commands";

export const useThemeSelector = (themes: ThemeItem[]) => {
  const [config, setConfig] = createSignal<Config>();
  const [appliedThemeID, setAppliedThemeID] = createSignal<string>();
  const [downloadThemeID, setDownloadThemeID] = createSignal<string>();
  const [index, setIndex] = createSignal(0);
  const [themeExists, setThemeExists] = createSignal(false);

  const currentTheme = createMemo(() => themes[index()]);

  const autoRun = async (config: Config) => {
    const { selected_theme_id, ...themeParams } = config;
    if (!selected_theme_id) return;

    await applyTheme({
      selected_theme_id,
      ...themeParams,
    });

    setAppliedThemeID(selected_theme_id);
    setIndex(themes.findIndex((t) => t.id === selected_theme_id));
  };

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
    setConfig((prev) => {
      const stoppedConfig = { ...prev!, selected_theme_id: undefined };
      applyTheme(stoppedConfig);
      return stoppedConfig;
    });
    setAppliedThemeID();
  };

  const onApply = async () => {
    const newConfig = {
      ...config()!,
      selected_theme_id: currentTheme().id,
    };
    await applyTheme(newConfig);
    setConfig(newConfig);
    setAppliedThemeID(newConfig.selected_theme_id);
  };

  return {
    config,
    setConfig,
    appliedThemeID,
    setAppliedThemeID,
    downloadThemeID,
    setDownloadThemeID,
    index,
    setIndex,
    themeExists,
    currentTheme,
    autoRun,
    onMenuItemClick,
    onCloseTask,
    onApply,
  };
};
