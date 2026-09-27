// Responsibility: Candidate-pool collage wall — reshuffles randomly when the pool changes and tiles the whole area;
//       "today's theme" is marked on its own tile instead of a fixed corner.
import { randomStore } from "@/store/random.store";
import { themeList } from "@/store/catalog.selectors";
import { t } from "@/i18n";
import { getRandomSelection } from "@/ipc";
import { createMemo, createResource, Show } from "solid-js";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";
import { collageLayout } from "./poolLayout";

export function PoolCollage() {
  const pool = createMemo(() =>
    themeList().filter((theme) => randomStore.selected.includes(theme.id)),
  );
  const [selection] = createResource(getRandomSelection);
  const todayId = createMemo(() =>
    selection.error ? undefined : selection()?.themeId,
  );

  const layout = createMemo(() => collageLayout(pool()));

  return (
    <div
      class="relative grid min-h-0 flex-1 grid-cols-4 gap-0.75 overflow-hidden rounded-2xl border border-border bg-muted shadow-[0_1px_3px_rgba(0,0,0,.08)] dark:border-white/10 dark:bg-black dark:shadow-[0_24px_60px_rgba(0,0,0,.45)]"
      style={{
        "grid-template-rows": `repeat(${layout().rows}, minmax(0,1fr))`,
      }}
    >
      {layout().widths.map((colSpan, i) => (
        <div
          class="relative overflow-hidden"
          style={{ "grid-column": `span ${colSpan}` }}
        >
          <ThemeThumbnail
            themeId={layout().themes[i].id}
            class="saturate-[.92] transition-transform duration-500 hover:scale-105"
          />
          <Show when={layout().themes[i].id === todayId()}>
            <div class="pointer-events-none absolute left-2 top-2 z-10 flex items-center gap-1.5 rounded-full border border-primary/40 bg-background/80 px-2 py-1 shadow-[0_2px_8px_rgba(0,0,0,.12)] backdrop-blur-md dark:shadow-[0_8px_24px_rgba(0,0,0,.5)]">
              <span class="size-1.5 rounded-full bg-success animate-bdot" />
              <span class="text-[10px] font-semibold text-foreground">
                {t("stage.stats.today")}
              </span>
            </div>
          </Show>
        </div>
      ))}
      <div class="pointer-events-none absolute inset-0 bg-linear-to-t from-background/85 via-background/15 to-transparent" />

      <div class="pointer-events-none absolute bottom-4 left-4.5 right-4.5 text-foreground">
        <div class="font-display text-[15px] font-bold">
          {t("stage.collageTitle")}
        </div>
        <div class="mt-0.75 font-mono text-[11px] text-muted-foreground">
          {t("stage.collageNote", { n: pool().length })}
        </div>
      </div>
    </div>
  );
}
