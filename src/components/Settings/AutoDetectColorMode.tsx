import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useContext } from "solid-js";
import { AppContext } from "~/context";
import { writeConfigFile } from "~/commands";

const AutoDetectColorMode = () => {
  const { config, refetchConfig } = useContext(AppContext)!;

  const onSwitchAutoDetectColorMode = async () => {
    await writeConfigFile({
      ...config()!,
      auto_detect_color_mode: !config()!.auto_detect_color_mode,
    });

    refetchConfig();
  };

  return (
    <SettingsItem label="自动切换暗色模式">
      <LazySwitch
        checked={config()!.auto_detect_color_mode}
        onChange={onSwitchAutoDetectColorMode}
      />
    </SettingsItem>
  );
};

export default AutoDetectColorMode;
