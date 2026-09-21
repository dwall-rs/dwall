/* ===== src/domain/themes.ts ===== */
// 职责：把 Rust 主题目录（catalog.store）适配为 UI 主题模型。
import { catalogStore } from "@/store/catalog.store";
import type { Theme } from "./types";

/** 主题列表（来自 Rust 目录）。 */
export const themeList = (): Theme[] =>
  catalogStore.themes.map((t) => ({ id: t.id, name: t.name }));

/** 按 id 取主题；id 为空或目录中不存在时回退到第一个目录主题。 */
export const themeById = (id: string | undefined): Theme => {
  const found = id ? catalogStore.themes.find((t) => t.id === id) : undefined;
  const theme = found ?? catalogStore.themes[0];
  return theme
    ? { id: theme.id, name: theme.name }
    : { id: "default", name: "—" };
};
