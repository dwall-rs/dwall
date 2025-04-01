import { applyTheme } from "~/commands";
import { useConfig, useMonitor, useTheme } from "~/contexts";

export const useTaskManager = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
  const { setAppliedThemeID } = useTheme();
  const { id: monitorID } = useMonitor();

  const handleTaskClosure = async () => {
    if (!config()) return;

    const monitor_specific_wallpapers = {
      ...config()?.monitor_specific_wallpapers,
    };

    delete monitor_specific_wallpapers[monitorID()!];

    const updatedConfig: Config = {
      ...config()!,
      selected_theme_id: undefined,
      monitor_specific_wallpapers,
    };

    try {
      await applyTheme(updatedConfig);
      refetchConfig();
      setAppliedThemeID(undefined);
    } catch (e) {
      console.error("Failed to close task:", e);
    }
  };

  return {
    handleTaskClosure,
  };
};
