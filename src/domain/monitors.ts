/* ===== src/domain/monitors.ts ===== */
// 职责：显示器初始数据。
import type { Monitor } from "./types";

export const INITIAL_MONITORS: Monitor[] = [
  { id: "all", name: "所有显示器", res: "统一", theme: "catalina", on: true },
  {
    id: "lenovo",
    name: "LenovoDisplay",
    res: "3840×2160",
    theme: "minya",
    on: true,
  },
  {
    id: "generic",
    name: "Generic Monitor",
    res: "1920×1080",
    theme: "mojave",
    on: true,
  },
];
