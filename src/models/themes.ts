// Responsibility: pure mapping from the Rust theme catalog to the UI theme model. No store/i18n access.
import type { CatalogTheme } from "@/ipc";
import type { Theme } from "./types";

/**
 * A catalog entry as the UI model.
 *
 * Only a catalog theme can need downloading: a custom theme exists on disk by
 * construction, and Rust decides whether a catalog theme's frames are there.
 */
const toTheme = (theme: CatalogTheme): Theme => ({
  id: theme.id,
  name: theme.name,
  needsDownload: theme.source === "catalog" && !theme.installed,
});

/** Theme list (from the Rust catalog). */
export const themeList = (themes: CatalogTheme[]): Theme[] =>
  themes.map(toTheme);

/**
 * Look up a theme by id:
 * - present in the catalog → return the catalog entry;
 * - id set but not in the catalog → use the id as the name (possibly a custom/installed theme);
 * - id empty → fall back to the catalog's first theme.
 */
export const themeById = (
  themes: CatalogTheme[],
  id: string | undefined,
): Theme => {
  if (id) {
    const found = themes.find((t) => t.id === id);
    // Not in the catalog means it cannot be downloaded from the library, whatever
    // is on disk: the stage checks the frames itself before offering anything.
    return found ? toTheme(found) : { id, name: id, needsDownload: false };
  }
  const first = themes[0];
  return first
    ? toTheme(first)
    : { id: "default", name: "—", needsDownload: false };
};
