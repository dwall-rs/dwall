import { LazyFlex } from "~/lazy";
import AutoStart from "./AutoStart";
import AutoDetectColorScheme from "./AutoDetectColorScheme";
import CoordinateSource from "./CoordinateSource";
import Interval from "./Interval";
import GithubMirror from "./GithubMirror";
import ThemesDirectory from "./ThemesDirectory";
import LockScreenWallpaperSwitch from "./LockScreenWallpaperSwitch";
import SettingsFooter from "./Footer";

const Settings = () => {
  return (
    <LazyFlex
      direction="column"
      style={{
        // width: appVars.contentWidth,
        // "box-sizing": "border-box",
        height: "100%",
        flex: 1,
      }}
      align="stretch"
      justify="stretch"
      paddingBottom="xs"
      paddingTop="xs"
    >
      <LazyFlex
        direction="column"
        gap="xl"
        style={{ flex: 15 }}
        align="stretch"
        justify="stretch"
      >
        <AutoStart />

        <AutoDetectColorScheme />

        <LockScreenWallpaperSwitch />

        <CoordinateSource />

        <Interval />

        <ThemesDirectory />

        <GithubMirror />
      </LazyFlex>

      <SettingsFooter />
    </LazyFlex>
  );
};

export default Settings;
