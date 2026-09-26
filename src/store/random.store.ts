// 职责：随机模式——候选池勾选 + 保存到配置（写 wallpaper_mode = Random 并重启引擎）。
import { createStore } from "solid-js/store";

import type { Config } from "@/domain/config";
import { applyWallpaperMode } from "./settings.store";

interface RandomState {
  selected: string[]; // 当前勾选（= 候选池）
  saved: string[]; // 配置快照
  saving: boolean;
  savedAt: number | null;
}

const [randomStore, setRandomStore] = createStore<RandomState>({
  selected: [],
  saved: [],
  saving: false,
  savedAt: null,
});

/** 用配置初始化候选池（配置 + 主题目录加载后调用）。 */
const syncFromConfig = (config: Config | null, allIds: string[]) => {
  const pool =
    config?.wallpaper_mode?.mode === "random"
      ? config.wallpaper_mode.pool
      : null;
  // 配置里的池是真值源；剔除已不在目录中的（已卸载）主题 id。
  const selected = pool ? pool.filter((id) => allIds.includes(id)) : allIds;
  setRandomStore({
    selected: [...selected],
    saved: [...selected],
    saving: false,
    savedAt: null,
  });
};

const toggle = (id: string) =>
  setRandomStore("selected", (s) =>
    s.includes(id) ? s.filter((x) => x !== id) : [...s, id],
  );

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
      saving: false,
      savedAt: Date.now(),
    }));
  } catch {
    setRandomStore("saving", false);
  }
};

const discard = () => setRandomStore("selected", [...randomStore.saved]);

/** 派生：是否存在未保存改动（按集合比较，忽略顺序）。 */
const isDirty = (s: RandomState): boolean => {
  if (s.selected.length !== s.saved.length) return true;
  const saved = new Set(s.saved);
  return s.selected.some((id) => !saved.has(id));
};

export { randomStore, syncFromConfig, toggle, save, discard, isDirty };
