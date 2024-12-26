import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { writeConfigFile } from "~/commands";
import { translate, translateErrorMessage } from "~/utils/i18n";
import { message } from "@tauri-apps/plugin-dialog";

const AutoDetectColorMode = () => {
  const { config, refetchConfig, translations } = useAppContext();

  const onSwitchAutoDetectColorMode = async () => {
    try {
      await writeConfigFile({
        ...config()!,
        auto_detect_color_mode: !config()!.auto_detect_color_mode,
      });
      refetchConfig();
    } catch (error) {
      message(
        translateErrorMessage(
          translations()!,
          "message-switch-auto-light-dark-mode-failed",
          error,
        ),
        { kind: "error" },
      );
    }
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
