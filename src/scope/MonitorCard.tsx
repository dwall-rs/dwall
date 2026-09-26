/* ===== src/components/scope/MonitorCard.tsx ===== */
// 职责：单显示器卡；allUnified 时整体禁用（结构性互斥的执行点）。
import { Switch } from "@/components/ui/switch";
import {
  fixedStore,
  isApplied,
  themeForMon,
  toggleApply,
  selectMon,
} from "@/store/fixed.store";
import { monitorById } from "@/domain/monitors";
import { themeById } from "@/domain/themes";
import { clsx } from "@/utils";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";
import { Monitor } from "lucide-solid";
import { createMemo } from "solid-js";

interface Props {
  id: string;
}

export function MonitorCard({ id }: Props) {
  const mon = createMemo(() => monitorById(id));
  const sel = createMemo(
    () => !fixedStore.allUnified && fixedStore.curMon === id,
  );
  const themeId = createMemo(() => themeForMon(id));
  const theme = createMemo(() => themeById(themeId()));

  return (
    <div
      onClick={() => !fixedStore.allUnified && selectMon(id)}
      class={clsx(
        "relative mb-2 flex cursor-pointer items-center gap-2.75 rounded-lg border p-2.75 transition-all duration-200",
        sel()
          ? "border-border-2 bg-secondary"
          : "border-transparent hover:translate-x-0.5 hover:bg-secondary",
        fixedStore.allUnified && "pointer-events-none opacity-40 saturate-50",
      )}
    >
      {sel() && (
        <span class="absolute bottom-2.5 left-0 top-2.5 w-0.75 rounded-sm bg-primary" />
      )}
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
          {mon()?.name ?? id}
        </div>
        <div class="mt-0.75 flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground">
          <span class="text-foreground/80">{theme().name}</span>
        </div>
      </div>
      <Switch
        size="sm"
        checked={isApplied(id)}
        onCheckedChange={() => void toggleApply(id)}
        onClick={(e) => e.stopPropagation()}
      />
    </div>
  );
}
