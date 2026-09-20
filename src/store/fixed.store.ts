// 职责：固定模式——统一/单独互斥、当前显示器、查看时段、每显示器主题、逐条「应用/停止」。
//       固定模式无全局脏态：提交粒度是「一套」，靠 applied 集合表达。
import { INITIAL_MONITORS } from "@/domain/monitors";
import { createStore } from "solid-js/store";

type StrMap = Record<string, string>;
type BoolMap = Record<string, boolean>;

const initThemes = (): StrMap =>
  Object.fromEntries(INITIAL_MONITORS.map((m) => [m.id, m.theme]));
const initOn = (): BoolMap =>
  Object.fromEntries(INITIAL_MONITORS.map((m) => [m.id, m.on]));

interface FixedState {
  allUnified: boolean; // 统一所有显示器（与单独设置结构性互斥）
  curMon: string; // 当前编辑的显示器 id
  selWallpaperId: string | null; // null = 预览当前匹配的那张
  monitorThemes: StrMap; // monId -> themeId
  monitorOn: BoolMap;
  applied: string[]; // 已「应用」的作用域 key 集合
}

const [fixedStore, setFixedStore] = createStore<FixedState>({
  allUnified: true,
  curMon: "all",
  selWallpaperId: null,
  monitorThemes: initThemes(),
  monitorOn: initOn(),
  applied: [],
});

const toggleUnified = () => {
  setFixedStore((s) => ({
    ...s,
    allUnified: !s.allUnified,
    curMon: !s.allUnified ? "all" : s.curMon,
    selWallpaperId: null,
  }));
};

const selectMon = (id: string) => {
  setFixedStore((s) => ({ ...s, curMon: id, selWallpaperId: null }));
};

const selectWallpaper = (selWallpaperId: string | null) => {
  setFixedStore("selWallpaperId", selWallpaperId);
};

const setMonitorTheme = (monId: string, themeId: string) => {
  setFixedStore("monitorThemes", (s) => ({ ...s, [monId]: themeId }));
  selectWallpaper(null);
};

const toggleMonitorOn = (monId: string) => {
  setFixedStore("monitorOn", (s) => ({ ...s, [monId]: !s[monId] }));
};

const toggleApply = (scopeKey: string) => {
  // TODO(ipc): 把「该作用域 + 当前主题」写配置并通知引擎（逐条提交，自动生效）
  setFixedStore("applied", (s) =>
    s.includes(scopeKey) ? s.filter((k) => k !== scopeKey) : [...s, scopeKey],
  );
};

export {
  fixedStore,
  toggleUnified,
  selectMon,
  selectWallpaper,
  setMonitorTheme,
  toggleMonitorOn,
  toggleApply,
};
