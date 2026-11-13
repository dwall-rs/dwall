// Responsibility: random mode — candidate pool selection + saving to the config (write wallpaper_mode = Random and restart the engine).
import { createStore } from "solid-js/store";

import type { Config } from "@/models/config";
import { seedPool } from "@/models/pool";
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
const syncFromConfig = (config: Config | null, availableIds: string[]) => {
  const mode = config?.wallpaper_mode;
  const pool = mode?.mode === "random" ? mode.pool : null;
  // Only themes whose frames are on disk can enter the pool; the library offers the
  // download for the rest (see `models/pool`).
  const selected = seedPool(pool, availableIds);
  setRandomStore({
    selected,
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

  // Snapshot before awaiting. The save button is disabled while the request is in
  // flight, but the tiles in the library are not: a theme toggled during the round
  // trip is not in the pool that was written, and recording the store's contents on
  // return marked it saved anyway — the row lost its dirty state and the change was
  // never written.
  const payload = [...randomStore.selected];

  try {
    await applyWallpaperMode({
      mode: "random",
      pool: payload,
    });
    setRandomStore((s) => ({
      ...s,
      saved: payload,
      applied: true,
      saving: false,
      savedAt: Date.now(),
    }));
  } catch {
    // `applyWallpaperMode` already reports the failure to the user.
    setRandomStore("saving", false);
  }
};

const discard = () => setRandomStore("selected", [...randomStore.saved]);

/** Align the "applied" flag with the last successfully saved configuration. */
const setApplied = (applied: boolean) => setRandomStore("applied", applied);

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
  setApplied,
  isDirty,
};
