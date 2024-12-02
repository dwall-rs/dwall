import { LazyFlex } from "~/lazy";
import AutoStart from "./AutoStart";
import AutoDetectColorMode from "./AutoDetectColorMode";
import CoordinateSource from "./CoordinateSource";
import Interval from "./Interval";
import GithubMirror from "./GithubMirror";

const Settings = () => {
  return (
    <LazyFlex
      direction="vertical"
      gap={24}
      style={{ width: "480px", height: "100%" }}
    >
      <AutoStart />

      <AutoDetectColorMode />

      <CoordinateSource />

      <Interval />

      <GithubMirror />
    </LazyFlex>
  );
};

export default Settings;
