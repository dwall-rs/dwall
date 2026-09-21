/* ===== src/components/layout/FixedCommit.tsx ===== */
// 职责：固定模式提交视图——只显示「作用域 + 已应用/未应用」，无保存按钮（提交靠每套的应用/停止）。
import { fixedStore } from "@/store/fixed.store";
import { monitorById } from "@/domain/monitors";
import { themeById } from "@/domain/themes";
import { clsx } from "@/utils";

export function FixedCommit() {
  const scopeKey = fixedStore.allUnified ? "all" : fixedStore.curMon;
  const mon = monitorById(scopeKey);
  const isApplied = fixedStore.applied.includes(scopeKey);

  return (
    <div class="flex items-center gap-2.5 text-[12.5px] text-muted-foreground">
      <span class="font-display font-bold text-foreground">
        {mon?.name ?? scopeKey}
      </span>
      <span
        class={clsx(
          "font-mono text-[12px]",
          isApplied ? "text-success" : "text-muted-foreground",
        )}
      >
        {isApplied
          ? `已应用 ${themeById(fixedStore.monitorThemes[scopeKey]).name}`
          : "未应用"}
      </span>
    </div>
  );
}
