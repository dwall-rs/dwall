// Responsibility: theme catalog and monitor list (from Rust). The UI shows them read-only and holds no business logic.

import { createStore } from "solid-js/store";

import { getMonitors, getThemeCatalog } from "@/ipc";
import type { CatalogTheme, DisplayMonitor } from "@/ipc";
import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { logger } from "@/utils";

const log = logger.child("catalog");

interface CatalogState {
  status: "idle" | "loading" | "ready" | "error";
  themes: CatalogTheme[];
  monitors: DisplayMonitor[];
  /**
   * Bumped whenever a theme finishes downloading.
   *
   * Views that read a theme's frames from disk (the stage) key on it, so a download
   * started anywhere — the library included — is visible everywhere it matters.
   */
  installedRevision: number;
}

const [catalogStore, setCatalogStore] = createStore<CatalogState>({
  status: "idle",
  themes: [],
  monitors: [],
  installedRevision: 0,
});

const load = async () => {
  setCatalogStore("status", "loading");
  try {
    const [themes, monitors] = await Promise.all([
      getThemeCatalog(),
      getMonitors(),
    ]);
    setCatalogStore({
      themes,
      monitors: Object.values(monitors),
      status: "ready",
    });
  } catch (e) {
    log.error("Failed to load catalog", e);
    setCatalogStore("status", "error");
    toast.error(t("common.operationFailed"), { description: String(e) });
  }
};

/**
 * Records that `themeId`'s frames are on disk now.
 *
 * Called by the download store on success, which is the only thing that can make a
 * theme newly installed without the catalog being reloaded.
 */
const markInstalled = (themeId: string) => {
  const index = catalogStore.themes.findIndex((theme) => theme.id === themeId);
  // A theme the catalog does not list can still have been downloaded (an id from the
  // config, say); the revision still has to move so the stage re-reads its frames.
  if (index >= 0) setCatalogStore("themes", index, "installed", true);
  setCatalogStore("installedRevision", (revision) => revision + 1);
};

export { catalogStore, load, markInstalled };
