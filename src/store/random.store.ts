// 职责：随机模式——候选池勾选 + 整份 profile 的三帧保存。dirty 由 selector 派生，不存 state。
import { THEMES } from "@/domain/themes";
import { createStore } from "solid-js/store";

interface RandomState {
  selected: string[]; // 当前勾选（= 候选池）
  saved: string[]; // 磁盘快照
  saving: boolean;
  savedAt: number | null;
}

const ALL_IDS = THEMES.map((t) => t.id);

const [randomStore, setRandomStore] = createStore<RandomState>({
  selected: [...ALL_IDS], // 默认全选
  saved: [...ALL_IDS],
  saving: false,
  savedAt: null,
});

const toggle = (id: string) =>
  setRandomStore("selected", (s) => ({
    selected: s.includes(id) ? s.filter((x) => x !== id) : [...s, id],
  }));

const save = async () => {
  if (randomStore.selected.length === 0 || randomStore.saving) return;
  setRandomStore("saving", true); // ② 写入中
  // TODO(ipc): 写随机配置文件到磁盘
  await new Promise((r) => setTimeout(r, 650));
  // TODO(ipc): 通知引擎进程重载配置
  setRandomStore((s) => ({
    ...s,
    saved: [...s.selected],
    saving: false,
    savedAt: Date.now(),
  })); // ③ 落盘
};

const discard = () => setRandomStore("selected", [...randomStore.saved]);

/** 派生：是否存在未保存改动。组件用 useRandomStore(isDirty)。 */
const isDirty = (s: RandomState): boolean => {
  if (s.selected.length !== s.saved.length) return true;
  for (let i = 0; i < s.selected.length; i++)
    if (s.selected[i] !== s.saved[i]) return true;
  return false;
};

export { randomStore, toggle, save, discard, isDirty };
