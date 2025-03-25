import {
  createSignal,
  createMemo,
  createResource,
  createEffect,
} from "solid-js";
import { getMonitors } from "~/commands";

/**
 * Monitor management Hook, used to handle display selection and related states
 * @param config Application configuration
 * @returns Display selection related states and methods
 */
export const useMonitorSelection = (config: () => Config | undefined) => {
  const [monitorInfoObject] = createResource(getMonitors);
  const [monitorID, setMonitorID] = createSignal<string>("all");

  // Get monitor list
  const monitors = createMemo(() => {
    const monitorIDs = Object.keys(monitorInfoObject() ?? {}).sort();

    return [
      { value: "all", label: "All" },
      ...monitorIDs.map((id) => ({
        value: monitorInfoObject()?.[id].device_path || "",
        label: monitorInfoObject()?.[id].friendly_name || "",
      })),
    ];
  });

  // Get monitor-specific theme configuration
  const monitorSpecificThemes = createMemo(() => {
    return Object.entries(config()?.monitor_specific_wallpapers ?? {}).sort(
      (a, b) => a[0].toLocaleLowerCase().localeCompare(b[0]),
    );
  });

  // Check if all monitors are using the same theme
  const monitorSpecificThemesIsSame = createMemo(() => {
    const themes = monitorSpecificThemes();
    if (themes.length <= 1) return true;
    return themes.every((value) => value[1] === themes[0][1]);
  });

  // Initialize monitor selection based on configuration
  createEffect(() => {
    if (!config()?.monitor_specific_wallpapers) return;

    const selectValue = monitorSpecificThemesIsSame()
      ? "all"
      : monitorSpecificThemes()[0][0];
    setMonitorID(selectValue);
  });

  // Handle monitor change
  const handleMonitorChange = (value: string) => {
    setMonitorID(value);
  };

  return {
    monitorID,
    setMonitorID,
    monitors,
    monitorSpecificThemes,
    monitorSpecificThemesIsSame,
    handleMonitorChange,
  };
};
