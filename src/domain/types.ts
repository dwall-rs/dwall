/* ===== src/domain/types.ts ===== */
// 职责：纯类型。太阳位置由高度角+方位角组成；壁纸携带原生宽高与对应太阳位置；主题壁纸数量可变。
import type { SolarPosition } from "@/ipc/types";

export type Mode = "fixed" | "random";
export type View = "main" | "settings";
export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

export type SceneType =
  | "island"
  | "lake"
  | "peak"
  | "dunes"
  | "ridge"
  | "coast"
  | "earth";

/** 太阳位置：高度角（地平线以上为正）与方位角（北=0 东=90 南=180 西=270）。 */
export type { SolarPosition };

export interface Wallpaper {
  id: string;
  w: number; // 原生宽
  h: number; // 原生高（比例不固定）
  solar: SolarPosition; // 这张壁纸对应的太阳位置
}

export interface Theme {
  id: string;
  name: string;
  type: SceneType;
  wallpapers: Wallpaper[]; // 数量可变
}

export interface Monitor {
  id: string;
  name: string;
  res: string;
  theme: string;
  on: boolean;
}

export function eqArr(a: readonly string[], b: readonly string[]): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
  return true;
}
