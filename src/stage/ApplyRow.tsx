/* ===== src/stage/ApplyRow.tsx ===== */
// 职责：固定模式的逐条提交——「应用此套/停止」就是固定模式的「保存」，写配置 + 重启引擎。
import { Button } from "@/components/ui/button";
import { fixedStore, isApplied, toggleApply } from "@/store/fixed.store";
import { monitorById } from "@/domain/monitors";
import { t } from "@/i18n";
import { ArrowRight, Square } from "lucide-solid";

export function ApplyRow() {
  const scopeKey = () => (fixedStore.allUnified ? "all" : fixedStore.curMon);
  const mon = () => monitorById(scopeKey());
  const applied = () => isApplied(scopeKey());

  return (
    <div class="flex shrink-0 items-center justify-center gap-3 pb-4 pt-1">
      <span class="font-mono text-[11.5px] text-muted-foreground">
        {t("stage.applyTo", { name: mon()?.name ?? scopeKey() })}
      </span>
      {applied() ? (
        <Button
          onClick={() => void toggleApply(scopeKey())}
          class="bg-destructive-dim px-7 py-2.5 text-[13.5px] text-destructive shadow-[inset_0_0_0_1px_hsl(var(--destructive)/.4)] hover:bg-destructive hover:text-destructive-foreground"
        >
          <Square class="size-3.5 fill-current" />
          {t("stage.stop")}
        </Button>
      ) : (
        <Button
          onClick={() => void toggleApply(scopeKey())}
          class="px-7 py-2.5 text-[13.5px]"
        >
          <ArrowRight class="size-3.5" />
          {t("stage.apply")}
        </Button>
      )}
    </div>
  );
}
