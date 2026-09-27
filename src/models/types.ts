// Responsibility: UI display types (business data is provided by Rust, see src/ipc).
import type { SolarPosition } from "@/ipc/types";

export type { SolarPosition };

export type Mode = "fixed" | "random";
export type View = "main" | "settings";
export type ThemeMode = "light" | "dark" | "system";
export type ResolvedTheme = "light" | "dark";

/** Theme (from the Rust catalog). */
export interface Theme {
  id: string;
  name: string;
}

/** Wallpaper: target solar angle (image shown as a thumbnail, provided by the theme catalog). */
export interface Wallpaper {
  index: number;
  solar: SolarPosition;
}

export interface Monitor {
  id: string;
  name: string;
}
