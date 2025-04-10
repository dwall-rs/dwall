import {
  createSignal,
  createMemo,
  createResource,
  createEffect,
} from "solid-js";
import { getMonitors } from "~/commands";
import { useConfig } from "~/contexts";
import { isSubset } from "~/utils/array";

export const useMonitorSelection = () => {
  const { data: config } = useConfig();

  const [monitorInfoObject] = createResource(getMonitors);
  const [monitorID, setMonitorID] = createSignal<string>("all");

  const originalMonitors = createMemo(() => {
    const monitorIDs = Object.keys(monitorInfoObject() ?? {}).sort();

    return monitorIDs.map((id) => ({
      value: monitorInfoObject()?.[id].device_path || "",
      label: monitorInfoObject()?.[id].friendly_name || "",
    }));
  });

  // Get monitor list
  const monitors = createMemo(() => [
    { value: "all", label: "All" },
    ...originalMonitors(),
  ]);

  // Get monitor-specific theme configuration
  const monitorSpecificThemes = createMemo(() => {
    return Object.entries(config()?.monitor_specific_wallpapers ?? {}).sort(
      (a, b) => a[0].toLocaleLowerCase().localeCompare(b[0]),
    );
  });

  // Check if all monitors are using the same theme
  const monitorSpecificThemesIsSame = createMemo(() => {
    const themes = monitorSpecificThemes();
    const monitorIDs = Object.keys(monitorInfoObject() ?? {}).sort();

    if (
      !isSubset(
        monitorIDs,
        themes.map((i) => i[0]),
      )
    )
      return false;

    return themes.every((value) => value[1] === themes[0][1]);
  });

  // Initialize monitor selection based on configuration
  createEffect(() => {
    if (!config()?.monitor_specific_wallpapers) return;

    const selectValue = monitorSpecificThemesIsSame()
      ? "all"
      : (monitorSpecificThemes()[0]?.[0] ?? "all");
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
