/* ===== src/components/scope/ScopeColumn.tsx ===== */
// 职责：左栏容器——标题随模式 + 渲染对应作用域面板。
import { uiStore } from "@/store/ui.store";
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { FixedScope } from "./FixedScope";
import { RandomScope } from "./RandomScope";
import { Show } from "solid-js";

export function ScopeColumn() {
  return (
    <div class="col-scope flex min-h-0 flex-col border-r border-border">
      <div class="flex items-baseline justify-between px-4.5 pb-2.5 pt-4">
        <span class="font-display text-[11px] font-bold uppercase tracking-[1.4px] text-muted-foreground">
          {t(uiStore.mode === "fixed" ? "scope.monitors" : "scope.pool")}
        </span>

        <Show when={uiStore.mode === "random"}>
          <span class="font-mono text-[11px] text-muted-foreground">
            {t("scope.selectedCount", {
              n: randomStore.selected.length,
              total: themeList().length,
            })}
          </span>
        </Show>
      </div>
      <div class="scrollbar-thin min-h-0 flex-1 overflow-y-auto px-3 pb-4.5 pt-1">
        {uiStore.mode === "fixed" ? <FixedScope /> : <RandomScope />}
      </div>
    </div>
  );
}
