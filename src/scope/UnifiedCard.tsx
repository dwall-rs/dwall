// Responsibility: "Unify all monitors" toggle card — enabling it locks the individual settings below.
import { fixedStore, toggleUnified as toggle } from "@/store/fixed.store";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { Switch } from "~/components/ui/switch";
import { Monitor } from "lucide-solid";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

export function UnifiedCard() {
  return (
    <div
      class={clsx(
        "relative mb-1.5 mt-0.5 rounded-lg border p-3.25 transition-all duration-200",
        fixedStore.allUnified
          ? "border-primary bg-primary/8 shadow-[0_8px_24px_-10px_var(--primary)]"
          : "border-border-2 bg-primary/8 hover:border-primary/40",
      )}
    >
      {/* The card toggles and carries a Switch of its own, so the click target is a
          button laid over the card rather than the card itself: a button may not
          contain another one, and this is what the keyboard can reach. */}
      <button
        type="button"
        aria-label={t("scope.unified")}
        aria-pressed={fixedStore.allUnified}
        onClick={toggle}
        class="absolute inset-0 cursor-pointer rounded-lg border border-transparent outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
      />
      <div class="flex items-center gap-2.5">
        <div class="grid size-7.5 shrink-0 place-items-center rounded-lg bg-accent text-primary">
          <Monitor class="size-4" />
        </div>
        <div class="flex-1 font-display text-[13.5px] font-bold">
          {t("scope.unified")}
        </div>
        <Switch
          class="z-10"
          size="sm"
          checked={fixedStore.allUnified}
          onCheckedChange={() => toggle()}
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
