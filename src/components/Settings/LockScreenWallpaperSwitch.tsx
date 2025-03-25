import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useContext } from "solid-js";
import { AppContext } from "~/context";
import { writeConfigFile } from "~/commands";
import { useTranslations } from "../TranslationsContext";

const LockScreenWallpaperSwitch = () => {
  const { config, refetchConfig } = useContext(AppContext)!;
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
    >
      <LazySwitch
        checked={config()!.lock_screen_wallpaper_enabled}
        onChange={onSwitchLockScreenWallpaper}
      />
    </SettingsItem>
  );
};

export default LockScreenWallpaperSwitch;
