/* ===== src/components/scope/ScopeColumn.tsx ===== */
// Responsibility: Left-column container — the monitor scope panel for fixed mode (random mode has no left column).
import { t } from "@/i18n";
import { FixedScope } from "./FixedScope";

export function ScopeColumn() {
  return (
    <div class="col-scope flex min-h-0 flex-col border-r border-border">
      <div class="flex items-baseline justify-between px-4.5 pb-2.5 pt-4">
        <span class="font-display text-[11px] font-bold uppercase tracking-[1.4px] text-muted-foreground">
          {t("scope.monitors")}
        </span>
      </div>
      <div class="min-h-0 flex-1 overflow-y-auto px-3 pb-4.5 pt-1">
        <FixedScope />
      </div>
    </div>
  );
}
