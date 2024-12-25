import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { writeConfigFile } from "~/commands";
import { translate } from "~/utils/i18n";

const AutoDetectColorMode = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const onSwitchAutoDetectColorMode = async () => {
    await writeConfigFile({
      ...config()!,
      auto_detect_color_mode: !config()!.auto_detect_color_mode,
    });

    refetchConfig();
  };

  return (
    <SettingsItem
      label={translate(
        translations()!,
        "label-automatically-switch-to-dark-mode",
      )}
    >
      <LazySwitch
        checked={config()!.auto_detect_color_mode}
        onChange={onSwitchAutoDetectColorMode}
      />
    </SettingsItem>
  );
};

export default AutoDetectColorMode;
