// 职责：视图路由态——view / mode / 主题库抽屉开关。切到设置视图时顺手收起抽屉。
import type { Mode, View } from "@/domain/types";
import { createStore } from "solid-js/store";

interface UiState {
  view: View;
  mode: Mode;
  libOpen: boolean; // 最小档下主题库抽屉是否展开（xl+ 忽略此值）
}

const [uiStore, setUiStore] = createStore<UiState>({
  view: "main",
  mode: "fixed",
  libOpen: false,
});

const setView = (view: View) =>
  setUiStore((prev) => ({ ...prev, view, libOpen: false }));

const toggleView = () =>
  setUiStore((prev) => ({
    ...prev,
    view: prev.view === "main" ? "settings" : "main",
    libOpen: false,
  }));

const setMode = (mode: Mode) => setUiStore("mode", mode);

const setLibOpen = (libOpen: boolean) => setUiStore("libOpen", libOpen);

const toggleLib = () => setUiStore("libOpen", (prev) => !prev);

export { uiStore, setView, toggleView, setMode, setLibOpen, toggleLib };
