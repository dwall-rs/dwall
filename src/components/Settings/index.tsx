import {
  LazyButton,
  LazyFlex,
  LazyLabel,
  LazySpace,
  LazyTooltip,
} from "~/lazy";
import { getVersion } from "@tauri-apps/api/app";
import AutoStart from "./AutoStart";
import AutoDetectColorMode from "./AutoDetectColorMode";
import CoordinateSource from "./CoordinateSource";
import Interval from "./Interval";
import GithubMirror from "./GithubMirror";
import { createResource } from "solid-js";
import { openConfigDir } from "~/commands";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { open } from "@tauri-apps/plugin-shell";
import { useAppContext } from "~/context";
import ThemesDirectory from "./ThemesDirectory";
import LockScreenWallpaperSwitch from "./LockScreenWallpaperSwitch";
import { translate } from "~/utils/i18n";
import { AiFillGithub } from "solid-icons/ai";

const Settings = () => {
  const {
    update: { resource, refetch, setShowDialog },
    translations,
  } = useAppContext();
  const [version] = createResource(getVersion);

  const onOpenLogDir = async () => {
    await openConfigDir();
  };

  const onUpdate = async () => {
    if (!resource()) {
      refetch();
    }
    const update = resource();
    if (!update) {
      await message(
        translate(translations()!, "message-version-is-the-latest"),
      );
      return;
    }

    const { currentVersion, version, body } = update;

    const result = await ask(
      `当前版本 ${currentVersion}，有新版本 ${version}。\n\n更新日志：\n\n${body}\n\n是否更新？`,
      "Dwall",
    );
    if (!result) return;
    setShowDialog(true);
  };

  const onOpenGithub = () => open("https://github.com/dwall-rs/dwall");

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

        <LazyFlex flex={1} justify="between">
          <LazySpace gap={8}>
            <LazyLabel>{translate(translations()!, "label-version")}</LazyLabel>

            <LazyTooltip
              content={translate(translations()!, "tooltip-check-new-version")}
              relationship="label"
              withArrow
            >
              <LazyButton
                appearance="transparent"
                style={{ "min-width": "48px" }}
                onClick={onUpdate}
              >
                {version()}
              </LazyButton>
            </LazyTooltip>
          </LazySpace>

          <LazySpace gap={8}>
            <LazyLabel>
              {translate(translations()!, "label-source-code")}
            </LazyLabel>

            <LazyButton
              appearance="subtle"
              icon={<AiFillGithub />}
              onClick={onOpenGithub}
            />
          </LazySpace>

          <LazySpace>
            <LazyButton appearance="subtle" onClick={onOpenLogDir}>
              {translate(translations()!, "button-open-log-directory")}
            </LazyButton>
          </LazySpace>
        </LazyFlex>
      </LazyFlex>
    </>
  );
};

export default Settings;
