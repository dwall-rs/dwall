// Responsibility: Center-column container — title follows the mode + renders the corresponding stage panel.
import { modeStore } from "@/store/mode.store";
import { scopeKey, themeForMon } from "@/store/fixed.store";
import { monitorById, themeById } from "@/store/catalog.selectors";
import { FixedStage } from "./FixedStage";
import { RandomStage } from "./RandomStage";
import { createMemo, Show } from "solid-js";

export function StageColumn() {
  const mon = createMemo(() => monitorById(scopeKey()));
  const themeName = createMemo(() => themeById(themeForMon(scopeKey())).name);

  return (
    <div class="col-stage flex min-h-0 flex-col gap-3 overflow-hidden px-4 pt-4 xl:px-6.5">
      <Show when={modeStore.mode === "fixed"}>
        <div class="flex items-center gap-2.5 text-[12.5px] text-muted-foreground">
          <span class="font-display text-[14px] font-bold text-foreground">
            {mon()?.name ?? scopeKey()}
          </span>
          <span>›</span>
          <span class="font-display font-semibold text-primary">
            {themeName()}
          </span>
        </div>
      </Show>

      <Show when={modeStore.mode === "fixed"} fallback={<RandomStage />}>
        <FixedStage />
      </Show>
    </div>
  );
}
