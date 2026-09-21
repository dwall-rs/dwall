// 职责：主题目录（来自 Rust）。UI 只读展示，不持有业务逻辑。

import { createStore } from "solid-js/store";

import { getThemeCatalog } from "@/ipc";
import type { CatalogTheme } from "@/ipc";
import { logger } from "@/utils";

const log = logger.child("catalog");

interface CatalogState {
  status: "idle" | "loading" | "ready" | "error";
  themes: CatalogTheme[];
}

const [catalogStore, setCatalogStore] = createStore<CatalogState>({
  status: "idle",
  themes: [],
});

const load = async () => {
  setCatalogStore("status", "loading");
  try {
    setCatalogStore({ themes: await getThemeCatalog(), status: "ready" });
  } catch (e) {
    log.error("Failed to load theme catalog", e);
    setCatalogStore("status", "error");
  }
};

export { catalogStore, load };
