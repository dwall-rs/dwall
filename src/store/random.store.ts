// 职责：随机模式——候选池勾选 + 保存到配置（写 wallpaper_mode = Random 并重启引擎）。
import { createStore } from "solid-js/store";

import type { Config } from "@/domain/config";
import { applyWallpaperMode } from "./settings.store";

interface RandomState {
  selected: string[]; // 当前勾选（= 候选池）
  saved: string[]; // 配置快照
  applied: boolean; // 配置里的 wallpaper_mode 是否已是 Random
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

/** 用配置初始化候选池（配置 + 主题目录加载后调用）。 */
const syncFromConfig = (config: Config | null, allIds: string[]) => {
  const mode = config?.wallpaper_mode;
  const pool = mode?.mode === "random" ? mode.pool : null;
  // 配置里的池是真值源；剔除已不在目录中的（已卸载）主题 id。
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

/** 全选（把候选池设为给定的全部主题 id）。 */
const selectAll = (ids: string[]) => setRandomStore("selected", [...ids]);

/** 取消全选（清空候选池）。 */
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

/** 派生：是否存在未保存改动（未应用随机模式，或候选池集合变化）。 */
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
