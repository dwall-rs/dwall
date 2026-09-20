/* ===== src/components/scope/MonitorCard.tsx ===== */
// 职责：单显示器卡；allUnified 时整体禁用（结构性互斥的执行点）。
import { Switch } from "@/components/ui/switch";
import {
  fixedStore,
  toggleMonitorOn as toggleOn,
  selectMon,
} from "@/store/fixed.store";
import { INITIAL_MONITORS } from "@/domain/monitors";
import { themeById } from "@/domain/themes";
import { clsx } from "@/utils";
import { ThemeCover } from "~/scene/ThemeCover";

interface Props {
  id: string;
}

export function MonitorCard({ id }: Props) {
  const mon = INITIAL_MONITORS.find((m) => m.id === id)!;
  const sel = !fixedStore.allUnified && fixedStore.curMon === id;

  return (
    <div
      onClick={() => !fixedStore.allUnified && selectMon(id)}
      class={clsx(
        "relative mb-2 flex cursor-pointer items-center gap-2.75 rounded-lg border p-2.75 transition-all duration-200",
        sel
          ? "border-border-2 bg-secondary"
          : "border-transparent hover:translate-x-0.5 hover:bg-secondary",
        fixedStore.allUnified && "pointer-events-none opacity-40 saturate-50",
      )}
    >
      {sel && (
        <span class="absolute bottom-2.5 left-0 top-2.5 w-0.75 rounded-sm bg-primary" />
      )}
      <div class="h-9.5 w-13.5 shrink-0 overflow-hidden rounded-[7px] shadow-[0_2px_8px_rgba(0,0,0,.45)]">
        <ThemeCover themeId={fixedStore.monitorThemes[id]} />
      </div>
      <div class="min-w-0 flex-1">
        <div class="truncate font-display text-[13.5px] font-bold">
          {mon.name}
        </div>
        <div class="mt-0.75 flex items-center gap-1.5 font-mono text-[11px] text-muted-foreground">
          <span class="text-foreground/80">
            {themeById(fixedStore.monitorThemes[id]).name}
          </span>
          · {mon.res}
        </div>
      </div>
      <Switch
        size="sm"
        checked={fixedStore.monitorOn[id]}
        onCheckedChange={() => toggleOn(id)}
        onClick={(e) => e.stopPropagation()}
      />
    </div>
  );
}
