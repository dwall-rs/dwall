import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useAppContext } from "~/context";
import { writeConfigFile } from "~/commands";
import { message } from "@tauri-apps/plugin-dialog";
import { useTranslations } from "~/contexts";

const AutoDetectColorMode = () => {
  const { config, refetchConfig } = useAppContext();
  const { translate, translateErrorMessage } = useTranslations();

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
          "message-switch-auto-light-dark-mode-failed",
          error,
        ),
        { kind: "error" },
      );
    }
  };

  return (
    <SettingsItem label={translate("label-automatically-switch-to-dark-mode")}>
      <LazySwitch
        checked={config()!.auto_detect_color_mode}
        onChange={onSwitchAutoDetectColorMode}
      />
    </SettingsItem>
  );
};

export default AutoDetectColorMode;
