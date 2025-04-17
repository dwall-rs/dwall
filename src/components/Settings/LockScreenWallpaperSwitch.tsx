import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";

import { writeConfigFile } from "~/commands";

import { useConfig, useTranslations } from "~/contexts";

const LockScreenWallpaperSwitch = () => {
  const { data: config, refetch: refetchConfig } = useConfig();
  const { translate } = useTranslations();

  const onSwitchLockScreenWallpaper = async () => {
    await writeConfigFile({
      ...config()!,
      lock_screen_wallpaper_enabled: !config()!.lock_screen_wallpaper_enabled,
    });

    refetchConfig();
  };

  return (
    <SettingsItem
      label={translate("label-set-lock-screen-wallpaper-simultaneously")}
      help={translate("help-set-lock-screen-wallpaper-simultaneously")}
    >
      <LazySwitch
        checked={config()!.lock_screen_wallpaper_enabled}
        onChange={onSwitchLockScreenWallpaper}
      />
    </SettingsItem>
  );
};

export default LockScreenWallpaperSwitch;
