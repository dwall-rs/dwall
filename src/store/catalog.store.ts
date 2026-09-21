// 职责：主题目录与显示器列表（来自 Rust）。UI 只读展示，不持有业务逻辑。

import { createStore } from "solid-js/store";

import { getMonitors, getThemeCatalog } from "@/ipc";
import type { CatalogTheme, DisplayMonitor } from "@/ipc";
import { logger } from "@/utils";

const log = logger.child("catalog");

interface CatalogState {
  status: "idle" | "loading" | "ready" | "error";
  themes: CatalogTheme[];
  monitors: DisplayMonitor[];
}

const [catalogStore, setCatalogStore] = createStore<CatalogState>({
  status: "idle",
  themes: [],
  monitors: [],
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
  }
};

export { catalogStore, load };
