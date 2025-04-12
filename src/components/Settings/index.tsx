import { LazyFlex } from "~/lazy";
import AutoStart from "./AutoStart";
import AutoDetectColorMode from "./AutoDetectColorMode";
import CoordinateSource from "./CoordinateSource";
import Interval from "./Interval";
import GithubMirror from "./GithubMirror";
import ThemesDirectory from "./ThemesDirectory";
import LockScreenWallpaperSwitch from "./LockScreenWallpaperSwitch";
import SettingsFooter from "./Footer";

import "./index.scss";

const Settings = () => {
  return (
    <>
      <LazyFlex direction="vertical" style={{ width: "480px", height: "100%" }}>
        <LazyFlex direction="vertical" gap={24} flex={15}>
          <AutoStart />

          <AutoDetectColorMode />

          <LockScreenWallpaperSwitch />

          <CoordinateSource />

          <Interval />

          <ThemesDirectory />

          <GithubMirror />
        </LazyFlex>

        <SettingsFooter />
      </LazyFlex>
    </>
  );
};

export default Settings;
