/* ===== src/domain/themes.ts ===== */
// 职责：主题数据。壁纸太阳位置改为「沿日轨曲线按不规则时刻采样」，保证标记落在曲线上；
//       数量可变、比例可变、不绑定命名时段。
import type { Theme, Wallpaper } from "./types";
import { nearestWallpaper, solarAt } from "./solar";

const RATIOS: [number, number][] = [
  [16, 10],
  [16, 9],
  [21, 9],
  [4, 3],
  [3, 2],
  [19.5, 9],
  [5, 4],
  [3, 4],
];
const rnd = (seed: number) => {
  let s = seed;
  return () => {
    s = (s * 9301 + 49297) % 233280;
    return s / 233280;
  };
};

function makeWallpapers(count: number, seed: number): Wallpaper[] {
  const rand = rnd(seed);
  const arr: Wallpaper[] = [];
  const span = 15; // 约 05:30–20:30 的日照区间
  for (let i = 0; i < count; i++) {
    // 不规则时刻（带 <1h 抖动），不落在固定"时段"上；solarAt 保证 (高度角,方位角) 在日轨曲线上
    const f = Math.min(
      21.5,
      Math.max(4.5, 5.5 + ((i + rand() * 0.9) / count) * span),
    );
    const [rw, rh] = RATIOS[(i + seed) % RATIOS.length];
    arr.push({
      id: `wp-${seed}-${i}`,
      w: rw * 100,
      h: rh * 100,
      solar: solarAt(f),
    });
  }
  return arr;
}
const mk = (
  id: string,
  name: string,
  type: Theme["type"],
  count: number,
  seed: number,
): Theme => ({ id, name, type, wallpapers: makeWallpapers(count, seed) });

export const THEMES: Theme[] = [
  mk("catalina", "Catalina", "island", 8, 11),
  mk("lake", "Lake-thep0y", "lake", 6, 17),
  mk("minya", "Minya-Konka", "peak", 9, 23),
  mk("bigsur", "Big Sur", "ridge", 7, 29),
  mk("mojave", "Mojave", "dunes", 5, 31),
  mk("monterey", "Monterey", "coast", 8, 37),
  mk("goldengate", "Golden Gate", "coast", 6, 41),
  mk("bluemarble", "Blue Marble", "earth", 4, 43),
  mk("ventura", "Ventura", "ridge", 7, 47),
  mk("sonoma", "Sonoma", "lake", 6, 53),
  mk("sequoia", "Sequoia", "peak", 9, 59),
  mk("sierra", "Sierra", "island", 5, 61),
];

export const themeById = (id: string): Theme =>
  THEMES.find((t) => t.id === id) ?? THEMES[0];
export const themeCover = (theme: Theme): Wallpaper =>
  nearestWallpaper(theme.wallpapers, { altitude: 42, azimuth: 180 });
