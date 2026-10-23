// Responsibility: Single-monitor card; the whole card is disabled when allUnified (the enforcement point of the structural exclusion).
import { Switch } from "@/components/ui/switch";
import {
  fixedStore,
  isApplied,
  themeForMon,
  toggleApply,
  selectMon,
} from "@/store/fixed.store";
import { monitorById, themeById } from "@/store/catalog.selectors";
import { clsx } from "@/utils";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";
import { Monitor } from "lucide-solid";
import { createMemo } from "solid-js";

interface Props {
  id: string;
}

export function MonitorCard(props: Props) {
  const mon = createMemo(() => monitorById(props.id));
  const sel = createMemo(
    () => !fixedStore.allUnified && fixedStore.curMon === props.id,
  );
  const themeId = createMemo(() => themeForMon(props.id));
  const theme = createMemo(() => themeById(themeId()));

  return (
    <div
      class={clsx(
        "relative mb-2 flex items-center gap-2.75 rounded-lg border p-2.75 transition-all duration-200",
        sel()
          ? "border-border-2 bg-secondary"
          : "border-transparent hover:translate-x-0.5 hover:bg-secondary",
        fixedStore.allUnified && "pointer-events-none opacity-40 saturate-50",
      )}
    >
      {sel() && (
        <span class="absolute bottom-2.5 left-0 top-2.5 w-0.75 rounded-sm bg-primary" />
      )}
      {/* The card selects a monitor and carries a Switch of its own, so the click target
          is a button laid over the card rather than the card itself: a button may not
          contain another one, and this is what the keyboard can reach. */}
      <button
        type="button"
        aria-label={mon()?.name ?? props.id}
        aria-pressed={sel()}
        disabled={fixedStore.allUnified}
        onClick={() => selectMon(props.id)}
        class="absolute inset-0 cursor-pointer rounded-lg border border-transparent outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring/50"
      />
      <div class="h-9.5 w-13.5 shrink-0 overflow-hidden rounded-[7px] shadow-[0_2px_8px_rgba(0,0,0,.45)]">
        <ThemeThumbnail
          themeId={themeId() ?? ""}
          fallback={
            <div class="grid h-full w-full place-items-center bg-card">
              <Monitor class="size-4 text-muted-foreground" />
            </div>
          }
        />
      </div>
      <div class="min-w-0 flex-1">
        <div class="truncate font-display text-[13.5px] font-bold">
          {mon()?.name ?? props.id}
        </div>
        <div class="mt-0.75 flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground">
          <span class="text-foreground/80">{theme().name}</span>
        </div>
      </div>
      <Switch
        class="z-10"
        size="sm"
        checked={isApplied(props.id)}
        disabled={fixedStore.allUnified}
        onCheckedChange={() => void toggleApply(props.id)}
      />
    </div>
  );
}
