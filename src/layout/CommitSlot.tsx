// 职责：顶栏右侧提交槽——仅固定模式显示「已应用/未应用」；随机模式不显示（信息在舞台）。
import { Show } from "solid-js";
import { settingsStore } from "@/store/settings.store";
import { uiStore } from "@/store/ui.store";
import { FixedCommit } from "./FixedCommit";

export function CommitSlot() {
  const random = () =>
    settingsStore.config?.wallpaper_mode?.mode === "random" ||
    uiStore.mode === "random";

  return (
    <Show when={!random()}>
      <FixedCommit />
    </Show>
  );
}
