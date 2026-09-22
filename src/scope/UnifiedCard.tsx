// 职责：「统一所有显示器」开关卡——开启即锁定下方单独设置。
import { fixedStore, toggleUnified as toggle } from "@/store/fixed.store";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { Switch } from "~/components/ui/switch";
import { Monitor } from "lucide-solid";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

export function UnifiedCard() {
  return (
    <div
      onClick={toggle}
      class={clsx(
        "relative mb-1.5 mt-0.5 cursor-pointer rounded-lg border p-3.25 transition-all duration-200",
        fixedStore.allUnified
          ? "border-primary bg-primary/8 shadow-[0_0_0_1px_hsl(var(--primary)),0_8px_24px_hsl(var(--primary)/.12)]"
          : "border-border-2 bg-primary/8 hover:border-primary/40",
      )}
    >
      <div class="flex items-center gap-2.5">
        <div class="grid size-7.5 shrink-0 place-items-center rounded-lg bg-accent text-primary">
          <Monitor class="size-4" />
        </div>
        <div class="flex-1 font-display text-[13.5px] font-bold">
          {t("scope.unified")}
        </div>
        <Switch
          size="sm"
          checked={fixedStore.allUnified}
          onCheckedChange={() => toggle()}
          onClick={(e) => e.stopPropagation()}
        />
      </div>
      <div class="mt-2 text-[11px] leading-relaxed text-muted-foreground">
        {fixedStore.allUnified ? t("scope.unifiedOn") : t("scope.unifiedOff")}
      </div>
      {fixedStore.allUnified && (
        <div class="mt-2.5 flex gap-1.5">
          <div class="h-6 w-8.5 overflow-hidden rounded-[5px]">
            <ThemeThumbnail themeId={fixedStore.monitorThemes.all} />
          </div>
        </div>
      )}
    </div>
  );
}
