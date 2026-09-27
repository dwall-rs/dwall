// Responsibility: Global bar orchestration; allows wrapping so it degrades gracefully at the smallest tier (flex-wrap), everything else unchanged.
import { uiStore } from "@/store/ui.store";
import { ModeToggle } from "./ModeToggle";
import { EngineStatus } from "./EngineStatus";
import { CommitSlot } from "./CommitSlot";
import { RightCluster } from "./RightCluster";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function GlobalBar() {
  const inMain = createMemo(() => uiStore.view === "main");

  return (
    <div class="flex min-h-15 shrink-0 flex-wrap items-center gap-x-4.5 gap-y-2 border-b border-border px-5 py-3">
      <div
        class={clsx(
          "transition-opacity duration-300",
          inMain() ? "opacity-100" : "pointer-events-none opacity-0",
        )}
      >
        <ModeToggle />
      </div>
      <EngineStatus />
      <div
        class={clsx(
          "ml-auto transition-opacity duration-300",
          inMain() ? "opacity-100" : "pointer-events-none opacity-0",
        )}
      >
        <CommitSlot />
      </div>
      <RightCluster />
    </div>
  );
}
