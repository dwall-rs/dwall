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
import { check, type Update } from "@tauri-apps/plugin-updater";
import { ask, message } from "@tauri-apps/plugin-dialog";
import UpdateDialog from "./UpdateDialog";

const Settings = () => {
  const [version] = createResource(getVersion);
  const [update, setUpdate] = createSignal<Update | null>(null);

  const onOpenLogDir = async () => {
    await openConfigDir();
  };

  const onUpdate = async () => {
    const update = await check();
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

    setUpdate(update);
  };

  return (
    <>
      <LazyFlex direction="vertical" style={{ width: "480px", height: "100%" }}>
        <LazyFlex direction="vertical" gap={24} flex={15}>
          <AutoStart />

          <AutoDetectColorMode />

          <CoordinateSource />

          <Interval />

          <GithubMirror />
        </LazyFlex>

        <LazyFlex flex={1} justify="between">
          <LazySpace gap={8}>
            <LazyLabel>版本号</LazyLabel>

            <LazyTooltip text="单击检测更新" placement="top">
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

      <Show when={update()}>
        <UpdateDialog update={update()!} />
      </Show>
    </>
  );
};

export default Settings;
