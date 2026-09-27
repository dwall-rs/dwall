// Responsibility: Empty state for the candidate pool — guides the user to keep at least one set.

import { t } from "@/i18n";
import { ImageOff } from "lucide-solid";

export function PoolEmpty() {
  return (
    <div class="relative flex min-h-0 flex-1 flex-col items-center justify-center gap-2.5 rounded-[18px] border border-dashed border-border-2 text-muted-foreground">
      <ImageOff class="size-10 opacity-40" />
      <div class="font-display text-[15px] font-bold text-foreground/80">
        {t("stage.emptyTitle")}
      </div>
      <div class="max-w-[300px] text-center text-[12px] leading-relaxed">
        {t("stage.emptyDesc")}
      </div>
    </div>
  );
}
