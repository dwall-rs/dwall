// Responsibility: Per-item commit for fixed mode — "apply this set / stop" is fixed mode's "save": it writes the config + restarts the engine.
import { Button } from "@/components/ui/button";
import { isApplied, scopeKey, toggleApply } from "@/store/fixed.store";
import { monitorById } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { ArrowRight, Square } from "lucide-solid";

export function ApplyRow() {
  const mon = () => monitorById(scopeKey());
  const applied = () => isApplied(scopeKey());

  return (
    <div class="flex shrink-0 items-center justify-center gap-3 pb-4 pt-1">
      <span class="font-mono text-[11.5px] text-muted-foreground">
        {t("stage.applyTo", { name: mon()?.name ?? scopeKey() })}
      </span>
      {applied() ? (
        <Button
          variant="destructive"
          onClick={() => void toggleApply(scopeKey())}
          class="px-7 py-2.5 text-[13.5px]"
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
