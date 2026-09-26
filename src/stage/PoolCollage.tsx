/* ===== src/stage/PoolCollage.tsx ===== */
// 职责：候选池拼贴墙——池变化时随机重排、铺满整个区域；标注「非抽中结果」。
import { randomStore } from "@/store/random.store";
import { themeList } from "@/domain/themes";
import { t } from "@/i18n";
import { createMemo } from "solid-js";
import { ThemeThumbnail } from "~/scene/ThemeThumbnail";

const COLS = 4;

/** 候选池内容 → 种子：同一池得到稳定排布，池一变就换一种排列。 */
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

/** mulberry32：可复现的伪随机数。 */
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
 * 每行的列宽（1 或 2），各行之和恰为 COLS——保证铺满整个拼贴区域。
 * 行数取 `ceil(n / COLS)`，行内格数尽量均分，宽度再随机分配到各格。
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

  const layout = createMemo(() => {
    const tiles = pool();
    const rand = makeRng(seedOf(tiles.map((theme) => theme.id)));
    return {
      rows: Math.ceil(tiles.length / COLS),
      widths: rowWidths(tiles.length, rand).flat(),
      themes: shuffled(tiles, rand),
    };
  });

  return (
    <div
      class="relative grid min-h-0 flex-1 grid-cols-4 gap-[3px] overflow-hidden rounded-[18px] border border-border-2 bg-black shadow-[0_24px_60px_rgba(0,0,0,.45)]"
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
      <div class="pointer-events-none absolute inset-0 bg-gradient-to-t from-black/72 via-transparent to-transparent" />
      <div class="pointer-events-none absolute bottom-4 left-[18px] right-[18px] text-white">
        <div class="font-display text-[15px] font-bold">
          {t("stage.collageTitle")}
        </div>
        <div class="mt-[3px] font-mono text-[11px] text-white/70">
          {t("stage.collageNote", { n: pool().length })}
        </div>
      </div>
    </div>
  );
}
