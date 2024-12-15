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
import { createResource, createSignal, Show } from "solid-js";
import { openConfigDir } from "~/commands";
import { ask, message } from "@tauri-apps/plugin-dialog";
import UpdateDialog from "./UpdateDialog";
import { useAppContext } from "~/context";
import ThemesDirectory from "./ThemesDirectory";
import LockScreenWallpaperSwitch from "./LockScreenWallpaperSwitch";

const Settings = () => {
  const {
    update: { resource, refetch },
  } = useAppContext();
  const [version] = createResource(getVersion);
  const [downloading, setDownloading] = createSignal(false);

  const onOpenLogDir = async () => {
    await openConfigDir();
  };

  const onUpdate = async () => {
    if (!resource()) {
      refetch();
    }
    const update = resource();
    if (!update) {
      await message("当前已经最新版");
      return;
    }

    const { currentVersion, version, body } = update;

    const result = await ask(
      `当前版本 ${currentVersion}，有新版本 ${version}。\n\n更新日志：\n\n${body}\n\n是否更新？`,
      "Dwall",
    );
    if (!result) return;
    setDownloading(true);
  };

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
            <LazyLabel>版本号</LazyLabel>

            <LazyTooltip content="单击检测更新" relationship="label">
              <LazyButton
                appearance="transparent"
                style={{ "min-width": "48px" }}
                onClick={onUpdate}
              >
                {version()}
              </LazyButton>
            </LazyTooltip>
          </LazySpace>

          <LazySpace>
            <LazyButton appearance="transparent" onClick={onOpenLogDir}>
              打开日志目录
            </LazyButton>
          </LazySpace>
        </LazyFlex>
      </LazyFlex>

      <Show when={downloading()}>
        <UpdateDialog update={resource()!} />
      </Show>
    </>
  );
};

export default Settings;
