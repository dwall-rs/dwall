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
  /**
   * Why the last load failed, for the library to show.
   *
   * The toast is long gone by the time the user looks at the library, so a failure has
   * to be readable where the themes would have been — and retryable from there.
   */
  error: string;
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
  error: "",
  themes: [],
  monitors: [],
  installedRevision: 0,
});

const load = async () => {
  setCatalogStore("status", "loading");
  // A download can finish while the catalog is being read. Its installed flag is set on
  // the list already in the store, which the fresh list would otherwise replace; the
  // revision tells us whether one did.
  const revisionBefore = catalogStore.installedRevision;
  try {
    const [themes, monitors] = await Promise.all([
      getThemeCatalog(),
      getMonitors(),
    ]);
    const installedDuringLoad =
      catalogStore.installedRevision === revisionBefore
        ? null
        : new Set(
            catalogStore.themes
              .filter((theme) => theme.installed)
              .map((theme) => theme.id),
          );
    setCatalogStore({
      themes: installedDuringLoad
        ? themes.map((theme) =>
            installedDuringLoad.has(theme.id)
              ? { ...theme, installed: true }
              : theme,
          )
        : themes,
      monitors: Object.values(monitors),
      status: "ready",
      error: "",
    });
  } catch (e) {
    log.error("Failed to load catalog", e);
    setCatalogStore({ status: "error", error: String(e) });
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
