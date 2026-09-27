// Responsibility: Candidate-pool collage wall — reshuffles randomly when the pool changes and tiles the whole area; a prominent badge shows "today's theme".
import { randomStore } from "@/store/random.store";
import { themeList, themeById } from "@/domain/themes";
import { t } from "@/i18n";
import { getRandomSelection } from "@/ipc";
import { createMemo, createResource, Show } from "solid-js";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

const COLS = 4;

/** Pool contents → seed: the same pool yields a stable layout; once the pool changes, so does the arrangement. */
function seedOf(ids: string[]): number {
  let hash = 2166136261;
  for (const id of [...ids].sort()) {
    for (let i = 0; i < id.length; i++) {
      hash ^= id.charCodeAt(i);
      hash = Math.imul(hash, 16777619);
    }
  }
  return hash >>> 0;
}

/** mulberry32: reproducible pseudo-random numbers. */
function makeRng(seed: number): () => number {
  let a = seed;
  return () => {
    a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

function shuffled<T>(items: T[], rand: () => number): T[] {
  const out = [...items];
  for (let i = out.length - 1; i > 0; i--) {
    const j = Math.floor(rand() * (i + 1));
    [out[i], out[j]] = [out[j], out[i]];
  }
  return out;
}

/**
 * Column widths per row (1 or 2), each row summing exactly to COLS — guarantees the entire collage area is filled.
 * Row count is `ceil(n / COLS)`, cells within a row are split as evenly as possible, then widths are distributed randomly.
 */
function rowWidths(n: number, rand: () => number): number[][] {
  const rows = Math.ceil(n / COLS);
  const counts: number[] = new Array(rows).fill(Math.floor(n / rows));
  for (let i = 0; i < n % rows; i++) counts[i] += 1;

  return counts.map((cells) => {
    const widths: number[] = new Array(cells).fill(Math.floor(COLS / cells));
    const order = shuffled(
      Array.from({ length: cells }, (_, i) => i),
      rand,
    );
    for (let i = 0; i < COLS % cells; i++) widths[order[i]] += 1;
    return widths;
  });
}

export function PoolCollage() {
  const pool = createMemo(() =>
    themeList().filter((theme) => randomStore.selected.includes(theme.id)),
  );
  const [selection] = createResource(getRandomSelection);

  const layout = createMemo(() => {
    const tiles = pool();
    const rand = makeRng(seedOf(tiles.map((theme) => theme.id)));
    return {
      rows: Math.max(1, Math.ceil(tiles.length / COLS)),
      widths: rowWidths(tiles.length, rand).flat(),
      themes: shuffled(tiles, rand),
    };
  });

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
