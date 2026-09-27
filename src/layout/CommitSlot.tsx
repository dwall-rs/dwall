// Responsibility: Commit slot on the right of the top bar — shows "applied / not applied" in fixed mode only; random mode shows nothing (the info lives on the stage).
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
