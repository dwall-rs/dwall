import { message, ask } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";

import { Switch } from "~/components/switch";
import SettingsItem from "./SettingsItem";

import { t } from "~/i18n";
import { useConfig } from "~/contexts";
import { writeConfigFile } from "~/commands";

const TitleBarColor = () => {
  const id = "title-bar-color";

  const { data: config, refetch: refetchConfig } = useConfig();

  const onSwitchTitleBarColorFollowsWindowsTheme = async () => {
    try {
      await writeConfigFile({
        ...config()!,
        title_bar_color_follows_windows_theme:
          !config()!.title_bar_color_follows_windows_theme,
      });

      const shouldRestart = await ask(
        t("settings.ask.titleBarColorFollowsWindowsTheme"),
      );
      shouldRestart ? relaunch() : refetchConfig();
    } catch (error) {
      message(
        t("settings.message.titleBarColorFollowsWindowsTheme", {
          error: String(error),
        }),
        { kind: "error" },
      );
    }
  };

  return (
    <SettingsItem
      label={t("settings.label.titleBarColorFollowsWindowsTheme")}
      help={t("settings.help.titleBarColorFollowsWindowsTheme")}
      for={id}
    >
      <Switch
        id={id}
        checked={config()!.title_bar_color_follows_windows_theme}
        onCheckedChange={onSwitchTitleBarColorFollowsWindowsTheme}
      />
    </SettingsItem>
  );
};

export default TitleBarColor;
