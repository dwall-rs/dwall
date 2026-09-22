/* ===== src/domain/themes.ts ===== */
// 职责：把 Rust 主题目录（catalog.store）适配为 UI 主题模型。
import { catalogStore } from "@/store/catalog.store";
import type { Theme } from "./types";

/** 主题列表（来自 Rust 目录）。 */
export const themeList = (): Theme[] =>
  catalogStore.themes.map((t) => ({ id: t.id, name: t.name }));

/**
 * 按 id 取主题：
 * - 目录中存在 → 返回目录项；
 * - id 有值但不在目录中 → 以 id 作为名称（可能是自定义/已安装主题）；
 * - id 为空 → 回退到目录首个主题。
 */
export const themeById = (id: string | undefined): Theme => {
  if (id) {
    const found = catalogStore.themes.find((t) => t.id === id);
    return found ? { id: found.id, name: found.name } : { id, name: id };
  }
  const first = catalogStore.themes[0];
  return first
    ? { id: first.id, name: first.name }
    : { id: "default", name: "—" };
};
