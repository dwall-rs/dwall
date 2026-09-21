/* ===== src/domain/themes.ts ===== */
// 职责：把 Rust 主题目录（catalog.store）适配为 UI 的主题模型。
//
// 说明：`type`（场景风格）与 `wallpapers`（壁纸太阳角）目前是「展示占位」，
// 由主题 id 确定性生成。真实壁纸太阳角接入后，改用 Rust 的 get_theme_wallpapers。
import { catalogStore } from "@/store/catalog.store";
import { nearestWallpaper, solarAt } from "./solar";
import type { SceneType, Theme, Wallpaper } from "./types";

const SCENE_TYPES: SceneType[] = [
  "island",
  "lake",
  "peak",
  "dunes",
  "ridge",
  "coast",
  "earth",
];
const RATIOS: [number, number][] = [
  [16, 10],
  [16, 9],
  [21, 9],
  [4, 3],
  [3, 2],
  [5, 4],
];

function hash(s: string): number {
  let h = 0;
  for (let i = 0; i < s.length; i++) h = (h * 31 + s.charCodeAt(i)) | 0;
  return Math.abs(h);
}

function placeholderWallpapers(themeId: string): Wallpaper[] {
  const seed = hash(themeId);
  const count = 6 + (seed % 4);
  const span = 15; // 约 05:30–20:30 的日照区间
  return Array.from({ length: count }, (_, i) => {
    const f = Math.min(
      21.5,
      Math.max(4.5, 5.5 + ((i + (seed % 7) * 0.13) / count) * span),
    );
    const [rw, rh] = RATIOS[(i + seed) % RATIOS.length];
    return {
      id: `wp-${themeId}-${i}`,
      w: rw * 100,
      h: rh * 100,
      solar: solarAt(f),
    };
  });
}

function toTheme(id: string, name: string): Theme {
  return {
    id,
    name,
    type: SCENE_TYPES[hash(id) % SCENE_TYPES.length],
    wallpapers: placeholderWallpapers(id),
  };
}

/** 主题列表（来自 Rust 目录）。 */
export const themeList = (): Theme[] =>
  catalogStore.themes.map((t) => toTheme(t.id, t.name));

/** 按 id 取主题；目录中不存在时回退为以该 id 命名的占位主题。 */
export const themeById = (id: string): Theme => {
  const found = catalogStore.themes.find((t) => t.id === id);
  return found ? toTheme(found.id, found.name) : toTheme(id, id);
};

/** 主题代表缩略（展示占位：取最接近正午的一张壁纸）。 */
export const themeCover = (theme: Theme): Wallpaper =>
  nearestWallpaper(theme.wallpapers, { altitude: 42, azimuth: 180 });
