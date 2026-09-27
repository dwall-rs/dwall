// Responsibility: random mode — candidate pool selection + saving to the config (write wallpaper_mode = Random and restart the engine).
import { createStore } from "solid-js/store";

import type { Config } from "@/domain/config";
import { applyWallpaperMode } from "./settings.store";

interface RandomState {
  selected: string[]; // current selection (= the candidate pool)
  saved: string[]; // config snapshot
  applied: boolean; // whether wallpaper_mode in the config is already Random
  saving: boolean;
  savedAt: number | null;
}

const [randomStore, setRandomStore] = createStore<RandomState>({
  selected: [],
  saved: [],
  applied: false,
  saving: false,
  savedAt: null,
});

/** Seed the candidate pool from the config (called once config + theme catalog are loaded). */
const syncFromConfig = (config: Config | null, allIds: string[]) => {
  const mode = config?.wallpaper_mode;
  const pool = mode?.mode === "random" ? mode.pool : null;
  // The pool in the config is the source of truth; drop theme ids no longer in the catalog (uninstalled).
  const selected = pool ? pool.filter((id) => allIds.includes(id)) : allIds;
  setRandomStore({
    selected: [...selected],
    saved: [...selected],
    applied: mode?.mode === "random",
    saving: false,
    savedAt: null,
  });
};

const toggle = (id: string) =>
  setRandomStore("selected", (s) =>
    s.includes(id) ? s.filter((x) => x !== id) : [...s, id],
  );

/** Select all (set the pool to every given theme id). */
const selectAll = (ids: string[]) => setRandomStore("selected", [...ids]);

/** Deselect all (empty the pool). */
const clearAll = () => setRandomStore("selected", []);

const save = async () => {
  if (randomStore.selected.length === 0 || randomStore.saving) return;
  setRandomStore("saving", true);
  try {
    await applyWallpaperMode({
      mode: "random",
      pool: [...randomStore.selected],
    });
    setRandomStore((s) => ({
      ...s,
      saved: [...s.selected],
      applied: true,
      saving: false,
      savedAt: Date.now(),
    }));
  } catch {
    setRandomStore("saving", false);
  }
};

const discard = () => setRandomStore("selected", [...randomStore.saved]);

/** Derived: whether unsaved changes exist (random mode not applied, or the pool set changed). */
const isDirty = (s: RandomState): boolean => {
  if (!s.applied) return true;
  if (s.selected.length !== s.saved.length) return true;
  const saved = new Set(s.saved);
  return s.selected.some((id) => !saved.has(id));
};

export {
  randomStore,
  syncFromConfig,
  toggle,
  selectAll,
  clearAll,
  save,
  discard,
  isDirty,
};
