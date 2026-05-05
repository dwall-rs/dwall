import AutoStart from "./AutoStart";
import AutoDetectColorScheme from "./AutoDetectColorScheme";
import CoordinateSource from "./CoordinateSource";
import Interval from "./Interval";
import GithubMirror from "./GithubMirror";
import ThemesDirectory from "./ThemesDirectory";
import LockScreenWallpaperSwitch from "./LockScreenWallpaperSwitch";
import SettingsFooter from "./Footer";
import Languages from "./Languages";

export const Settings = () => {
  return (
    <div class="flex flex-col items-stretch h-screen pb-1.5 px-1.5">
      <div class="flex flex-col flex-1 gap-3.5 px-2 py-1.5 overflow-y-auto overflow-x-hidden scrollbar">
        <Languages />

        <AutoStart />

        <AutoDetectColorScheme />

        <LockScreenWallpaperSwitch />

        <CoordinateSource />

        <Interval />

        <ThemesDirectory />

        <GithubMirror />
      </div>

      <SettingsFooter />
    </div>
  );
};
