// Responsibility: adapt the Rust theme catalog (catalog.store) to the UI theme model.
import { catalogStore } from "@/store/catalog.store";
import type { Theme } from "./types";

/** Theme list (from the Rust catalog). */
export const themeList = (): Theme[] =>
  catalogStore.themes.map((t) => ({ id: t.id, name: t.name }));

/**
 * Look up a theme by id:
 * - present in the catalog → return the catalog entry;
 * - id set but not in the catalog → use the id as the name (possibly a custom/installed theme);
 * - id empty → fall back to the catalog's first theme.
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
