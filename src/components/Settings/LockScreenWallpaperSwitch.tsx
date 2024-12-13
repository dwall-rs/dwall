import { LazySwitch } from "~/lazy";
import SettingsItem from "./item";
import { useContext } from "solid-js";
import { AppContext } from "~/context";
import { writeConfigFile } from "~/commands";

const LockScreenWallpaperSwitch = () => {
  const { config, refetchConfig } = useContext(AppContext)!;

  const onSwitchLockScreenWallpaper = async () => {
    await writeConfigFile({
      ...config()!,
      lock_screen_wallpaper_enabled: !config()!.lock_screen_wallpaper_enabled,
    });

    refetchConfig();
  };

  return (
    <SettingsItem label="同时设置锁屏壁纸">
      <LazySwitch
        checked={config()!.lock_screen_wallpaper_enabled}
        onChange={onSwitchLockScreenWallpaper}
      />
    </SettingsItem>
  );
};

export default LockScreenWallpaperSwitch;
