// Responsibility: pure layout math for the random-mode pool collage (stable shuffle + full-bleed tiling). No reactivity/DOM.
import type { Theme } from "@/domain/types";

export const COLS = 4;

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

export interface CollageLayout {
  rows: number;
  /** Grid column span per tile, in render order. */
  widths: number[];
  /** Tiles in render order. */
  themes: Theme[];
}

/** Deterministic, size-adaptive collage layout for the given pool. */
export function collageLayout(themes: Theme[]): CollageLayout {
  const rand = makeRng(seedOf(themes.map((theme) => theme.id)));
  return {
    rows: Math.max(1, Math.ceil(themes.length / COLS)),
    widths: rowWidths(themes.length, rand).flat(),
    themes: shuffled(themes, rand),
  };
}
