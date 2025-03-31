import { message } from "@tauri-apps/plugin-dialog";

import SettingsItem from "./item";
import { LazySwitch } from "~/lazy";

import { writeConfigFile } from "~/commands";

import { useConfig, useTranslations } from "~/contexts";

const AutoDetectColorMode = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
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
