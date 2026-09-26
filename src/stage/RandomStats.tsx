/* ===== src/components/stage/RandomStats.tsx ===== */
// 职责：随机模式只读摘要数字——候选池/范围/目标/周期。
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function RandomStats() {
  const n = createMemo(() => randomStore.selected.length);
  const empty = createMemo(() => n() === 0);
  return (
    <div class="flex flex-wrap gap-[30px]">
      <Stat
        value={String(n)}
        label={t("stage.stats.pool")}
        cls={empty() ? "text-destructive" : "text-warning"}
      />
      <Stat
        value={
          n() === themeList().length
            ? t("stage.stats.all")
            : t("stage.stats.exclude", { n: themeList().length - n() })
        }
        label={t("stage.stats.range")}
        small
        cls={empty() ? "text-destructive" : ""}
      />
      <Stat
        value={t("stage.stats.targetValue")}
        label={t("stage.stats.target")}
        cls="text-primary"
      />
      <Stat
        value={t("stage.stats.periodValue")}
        label={t("stage.stats.period")}
        small
      />
    </div>
  );
}
function Stat(props: {
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
          props.small ? "text-[24px]" : "text-[40px]",
          props.cls,
        )}
      >
        {props.value}
      </div>
      <div class="mt-1.5 font-mono text-[11px] tracking-wide text-muted-foreground">
        {props.label}
      </div>
    </div>
  );
}
