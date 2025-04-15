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
      <LazyFlex
        direction="column"
        style={{
          width: "var(--content-width)",
          "box-sizing": "border-box",
          height: "100%",
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
