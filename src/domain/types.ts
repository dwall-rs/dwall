/* ===== src/domain/types.ts ===== */
// 职责：UI 展示类型（业务数据由 Rust 提供，见 src/ipc）。
import type { SolarPosition } from "@/ipc/types";

export type { SolarPosition };

export type Mode = "fixed" | "random";
export type View = "main" | "settings";
export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

/** 主题（来自 Rust 目录）。 */
export interface Theme {
  id: string;
  name: string;
}

/** 壁纸：目标太阳角（图片以缩略图展示，由主题目录提供）。 */
export interface Wallpaper {
  index: number;
  solar: SolarPosition;
}

export interface Monitor {
  id: string;
  name: string;
}
