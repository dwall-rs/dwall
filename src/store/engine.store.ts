// 职责：引擎进程生命周期——运行/停止 + 终止的二次确认状态机。

import { createStore } from "solid-js/store";

import { getEngineStatus, startEngine, stopEngine } from "@/ipc";
import { logger } from "@/utils";

const log = logger.child("engine");

interface EngineState {
  running: boolean;
  pending: boolean; // 已点一次「终止」，等待二次确认
  pendingTimer: ReturnType<typeof setTimeout> | undefined;
}

const [engineStore, setEngineStore] = createStore<EngineState>({
  running: false,
  pending: false,
  pendingTimer: undefined,
});

/** 与真实守护进程状态对齐 */
const refresh = async () => {
  try {
    setEngineStore("running", await getEngineStatus());
  } catch (e) {
    log.error("Failed to query engine status", e);
  }
};

const toggle = async () => {
  if (engineStore.running) {
    if (engineStore.pending) {
      // 二次确认 → 真正终止
      if (engineStore.pendingTimer) clearTimeout(engineStore.pendingTimer);
      try {
        await stopEngine();
        setEngineStore({
          running: false,
          pending: false,
          pendingTimer: undefined,
        });
      } catch (e) {
        log.error("Failed to stop engine", e);
        setEngineStore((prev) => ({
          ...prev,
          pending: false,
          pendingTimer: undefined,
        }));
      }
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
    try {
      await startEngine();
      setEngineStore("running", true);
    } catch (e) {
      log.error("Failed to start engine", e);
    }
  }
};

export { engineStore, toggle, refresh };
