// Responsibility: Candidate-pool collage wall — reshuffles randomly when the pool changes and tiles the whole area; a prominent badge shows "today's theme".
import { randomStore } from "@/store/random.store";
import { themeList, themeById } from "@/domain/themes";
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

  const layout = createMemo(() => collageLayout(pool()));

  return (
    <div
      class="relative grid min-h-0 flex-1 grid-cols-4 gap-0.75 overflow-hidden rounded-2xl border border-border-2 bg-black shadow-[0_24px_60px_rgba(0,0,0,.45)]"
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
        </div>
      ))}
      <div class="pointer-events-none absolute inset-0 bg-linear-to-t from-black/72 via-transparent to-transparent" />

      {/* Today's theme badge */}
      <Show when={selection()}>
        {(sel) => (
          <div class="pointer-events-none absolute left-4.5 top-4 flex items-center gap-2 rounded-full border border-primary/40 bg-black/65 px-3.5 py-1.5 shadow-[0_8px_24px_rgba(0,0,0,.5)] backdrop-blur-md">
            <span class="size-1.5 rounded-full bg-success animate-bdot" />
            <span class="font-mono text-[10px] tracking-wide text-white/65">
              {t("stage.stats.today")}
            </span>
            <span class="font-display text-[13px] font-bold text-white">
              {themeById(sel().themeId).name}
            </span>
          </div>
        )}
      </Show>

      <div class="pointer-events-none absolute bottom-4 left-4.5 right-4.5 text-white">
        <div class="font-display text-[15px] font-bold">
          {t("stage.collageTitle")}
        </div>
        <div class="mt-0.75 font-mono text-[11px] text-white/70">
          {t("stage.collageNote", { n: pool().length })}
        </div>
      </div>
    </div>
  );
}
