/* ===== src/components/stage/RandomStats.tsx ===== */
// 职责：随机模式只读摘要数字——候选池/范围/目标/周期。
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function RandomStats() {
  const n = createMemo(() => randomStore.selected.length);
  const empty = createMemo(() => n() === 0);
  return (
    <div class="flex flex-wrap gap-[30px]">
      <Stat
        value={String(n)}
        label="候选池 / 套"
        cls={empty() ? "text-destructive" : "text-warning"}
      />
      <Stat
        value={
          n() === themeList().length
            ? "全部"
            : `排除 ${themeList().length - n()}`
        }
        label="范围"
        small
        cls={empty() ? "text-destructive" : ""}
      />
      <Stat value="ALL" label="目标显示器" cls="text-primary" />
      <Stat value="24h" label="切换周期" small />
    </div>
  );
}
function Stat({
  value,
  label,
  cls,
  small,
}: {
  value: string;
  label: string;
  cls?: string;
  small?: boolean;
}) {
  return (
    <div>
      <div
        class={clsx(
          "font-display font-extrabold leading-none tracking-tight",
          small ? "text-[24px]" : "text-[40px]",
          cls,
        )}
      >
        {value}
      </div>
      <div class="mt-1.5 font-mono text-[11px] tracking-wide text-muted-foreground">
        {label}
      </div>
    </div>
  );
}
