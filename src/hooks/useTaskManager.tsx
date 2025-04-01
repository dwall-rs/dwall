import { applyTheme } from "~/commands";
import { useConfig, useTheme } from "~/contexts";

export const useTaskManager = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
  const { setAppliedThemeID } = useTheme();

  const handleTaskClosure = async () => {
    if (!config()) return;

    const updatedConfig: Config = {
      ...config()!,
      selected_theme_id: undefined,
      monitor_specific_wallpapers: {},
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
