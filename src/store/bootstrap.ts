// Responsibility: the load of settings + catalog and the derivation of the editing
// stores from them, in one place that a retry can call again.
//
// The shell used to do this inline once. Both loads swallow their own errors, so when
// the catalog read failed the random pool was seeded from an empty available set and
// marked saved; the retry in the library only reloaded the catalog and never re-ran
// the derivation. The pool then showed zero members with no unsaved state, and saving
// it would overwrite the configured pool.

import { load as loadCatalog, catalogStore } from "./catalog.store";
import { availableThemeIds } from "./catalog.selectors";
import { syncFromConfig as syncFixed } from "./fixed.store";
import { setMode } from "./mode.store";
import { syncFromConfig as syncRandom } from "./random.store";
import { load as loadSettings, settingsStore } from "./settings.store";

/**
 * Re-derives the editing stores from the configuration that is loaded now.
 *
 * Runs after every successful settings or catalog load: the fixed mapping needs the
 * monitor list, and the random pool needs the themes that are actually on disk.
 */
export const syncDerivedState = async (): Promise<void> => {
  const config = settingsStore.config;

  await syncFixed(config);
  syncRandom(config, availableThemeIds());

  if (config?.wallpaper_mode?.mode === "random") setMode("random");
};

/** First load of the window: settings and catalog, then the editing state. */
export const bootstrap = async (): Promise<void> => {
  await Promise.all([loadSettings(), loadCatalog()]);
  await syncDerivedState();
};

/** Retry from the settings error panel, re-deriving only when the read succeeded. */
export const reloadSettings = async (): Promise<void> => {
  await loadSettings();

  if (settingsStore.status === "ready") await syncDerivedState();
};

/** Retry from the library error panel, re-deriving only when the read succeeded. */
export const reloadCatalog = async (): Promise<void> => {
  await loadCatalog();

  if (catalogStore.status === "ready") await syncDerivedState();
};
