import { message } from "@tauri-apps/plugin-dialog";

import SettingsItem from "./SettingsItem";
import { Switch } from "~/components/switch";

import { writeConfigFile } from "~/commands";

import { useConfig } from "~/contexts";
import { t } from "~/i18n";

const AutoDetectColorMode = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const onSwitchAutoDetectColorMode = async () => {
    try {
      await writeConfigFile({
        ...config()!,
        auto_detect_color_scheme: !config()!.auto_detect_color_scheme,
      });
      refetchConfig();
    } catch (error) {
      message(
        t("settings.message.switchAutoModesFailed", { error: String(error) }),
        { kind: "error" },
      );
    }
  };

  return (
    <SettingsItem
      label={t("settings.label.automaticallySwitchModes")}
      help={t("settings.help.automaticallySwitchModes")}
    >
      <Switch
        checked={config()!.auto_detect_color_scheme}
        onCheckedChange={onSwitchAutoDetectColorMode}
      />
    </SettingsItem>
  );
};

export default AutoDetectColorMode;
