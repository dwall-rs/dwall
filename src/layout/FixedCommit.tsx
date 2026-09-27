/* ===== src/layout/FixedCommit.tsx ===== */
// Responsibility: Fixed-mode commit view — shows only "scope + applied / not applied", with no save button (committing is done per-theme via apply/stop).
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
