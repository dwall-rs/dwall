// Responsibility: Read-only summary figures for random mode — candidate pool / range / target / period.
import { randomStore } from "@/store/random.store";
import { availableThemes } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { clsx } from "@/utils";
import { createMemo } from "solid-js";

export function RandomStats() {
  const n = createMemo(() => randomStore.selected.length);
  // The denominator is what can be applied: an uninstalled theme is not a candidate at
  // all, so it is neither pooled nor "excluded".
  const selectable = createMemo(() => availableThemes().length);
  const empty = createMemo(() => n() === 0);
  // With no installed themes, "0 of 0" is not "all"; and a selected theme that has
  // since been removed can make n greater than the denominator.
  const all = createMemo(() => selectable() > 0 && n() === selectable());
  const excluded = createMemo(() => Math.max(0, selectable() - n()));
  return (
    <div class="flex flex-wrap gap-7.5">
      <Stat
        value={String(n())}
        label={t("stage.stats.pool")}
        cls={empty() ? "text-destructive" : "text-warning"}
      />
      <Stat
        value={
          all()
            ? t("stage.stats.all")
            : t("stage.stats.exclude", { n: excluded() })
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
