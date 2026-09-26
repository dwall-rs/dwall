/* ===== src/layout/FixedCommit.tsx ===== */
// 职责：固定模式提交视图——只显示「作用域 + 已应用/未应用」，无保存按钮（提交靠每套的应用/停止）。
import { fixedStore, isApplied } from "@/store/fixed.store";
import { monitorById } from "@/domain/monitors";
import { themeById } from "@/domain/themes";
import { t } from "@/i18n";
import { clsx } from "@/utils";

export function FixedCommit() {
  const scopeKey = () => (fixedStore.allUnified ? "all" : fixedStore.curMon);
  const mon = () => monitorById(scopeKey());
  const applied = () => isApplied(scopeKey());

  return (
    <div class="flex items-center gap-2.5 text-[12.5px] text-muted-foreground">
      <span class="font-display font-bold text-foreground">
        {mon()?.name ?? scopeKey()}
      </span>
      <span
        class={clsx(
          "font-mono text-[12px]",
          applied() ? "text-success" : "text-muted-foreground",
        )}
      >
        {applied()
          ? t("commit.applied", {
              theme: themeById(fixedStore.monitorThemes[scopeKey()]).name,
            })
          : t("commit.notApplied")}
      </span>
    </div>
  );
}
