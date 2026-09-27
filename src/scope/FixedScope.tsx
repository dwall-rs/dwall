// Responsibility: Fixed-mode scope panel — unified card + divider + individual monitor list.
import { fixedStore } from "@/store/fixed.store";
import { monitorList } from "@/domain/monitors";
import { t } from "@/i18n";
import { UnifiedCard } from "./UnifiedCard";
import { MonitorCard } from "./MonitorCard";
import { clsx } from "@/utils";

export function FixedScope() {
  return (
    <>
      <UnifiedCard />
      <div
        class={clsx(
          "flex items-center gap-2 px-1.5 pb-1.5 pt-3 font-display text-[10px] font-bold uppercase tracking-[1.4px] text-muted-foreground",
          fixedStore.allUnified && "opacity-40",
        )}
      >
        {t("scope.individual")}
        <span class="h-px flex-1 bg-border" />
      </div>
      {monitorList()
        .filter((m) => m.id !== "all")
        .map((m) => (
          <MonitorCard id={m.id} />
        ))}
    </>
  );
}
