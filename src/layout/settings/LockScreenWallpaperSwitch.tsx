import { Switch } from "~/components/switch";
import SettingsItem from "./SettingsItem";

import { writeConfigFile } from "~/commands";

import { useConfig } from "~/contexts";
import { t } from "~/i18n";

const LockScreenWallpaperSwitch = () => {
  const { data: config, refetch: refetchConfig } = useConfig();

  const onSwitchLockScreenWallpaper = async () => {
    await writeConfigFile({
      ...config()!,
      lock_screen_wallpaper_enabled: !config()!.lock_screen_wallpaper_enabled,
    });

    refetchConfig();
  };

  return (
    <SettingsItem
      label={t("settings.label.setLockScreenWallpaperSimultaneously")}
      help={t("settings.help.setLockScreenWallpaperSimultaneously")}
    >
      <Switch
        checked={config()!.lock_screen_wallpaper_enabled}
        onCheckedChange={onSwitchLockScreenWallpaper}
      />
    </SettingsItem>
  );
};

export default LockScreenWallpaperSwitch;
