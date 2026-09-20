// 职责：引擎进程生命周期——运行/停止 + 终止的二次确认状态机。

import { createStore } from "solid-js/store";

interface EngineState {
  running: boolean;
  pending: boolean; // 已点一次「终止」，等待二次确认
  pendingTimer: ReturnType<typeof setTimeout> | undefined;
}

const [engineStore, setEngineStore] = createStore<EngineState>({
  running: true,
  pending: false,
  pendingTimer: undefined,
});

const toggle = () => {
  if (engineStore.running) {
    if (engineStore.pending) {
      // 二次确认 → 真正终止
      if (engineStore.pendingTimer) clearTimeout(engineStore.pendingTimer);
      // TODO(ipc): 向引擎进程发送终止信号 / kill pid
      setEngineStore({
        running: false,
        pending: false,
        pendingTimer: undefined,
      });
    } else {
      // 第一次点击 → 进入待确认，2.2s 不复位则自动放弃
      const t = setTimeout(
        () =>
          setEngineStore((prev) => ({
            ...prev,
            pending: false,
            pendingTimer: undefined,
          })),
        2200,
      );
      setEngineStore((prev) => ({ ...prev, pending: true, pendingTimer: t }));
    }
  } else {
    // TODO(ipc): 启动引擎进程
    setEngineStore("running", true);
  }
};

export { engineStore, toggle };
