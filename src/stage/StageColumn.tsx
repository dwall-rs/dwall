/* ===== src/components/stage/StageColumn.tsx ===== */
// 职责：中栏容器——标题随模式 + 渲染对应舞台面板。
import { uiStore } from "@/store/ui.store";
import { fixedStore } from "@/store/fixed.store";
import { INITIAL_MONITORS } from "@/domain/monitors";
import { themeById } from "@/domain/themes";
import { FixedStage } from "./FixedStage";
import { RandomStage } from "./RandomStage";
import { createMemo, Show } from "solid-js";

export function StageColumn() {
  const scopeKey = createMemo(() =>
    fixedStore.allUnified ? "all" : fixedStore.curMon,
  );
  const mon = createMemo(
    () => INITIAL_MONITORS.find((m) => m.id === scopeKey())!,
  );
  const themeName = createMemo(
    () => themeById(fixedStore.monitorThemes[scopeKey()]).name,
  );

  return (
    <div class="col-stage flex min-h-0 flex-col gap-3 px-4 pt-4 xl:px-6.5">
      <Show when={uiStore.mode === "fixed"}>
        <div class="flex items-center gap-2.5 text-[12.5px] text-muted-foreground">
          <span class="font-display text-[14px] font-bold text-foreground">
            {mon().name}
          </span>
          <span>›</span>
          <span class="font-display font-semibold text-primary">
            {themeName()}
          </span>
        </div>
      </Show>

      <Show when={uiStore.mode === "fixed"} fallback={<RandomStage />}>
        <FixedStage />
      </Show>
    </div>
  );
}
