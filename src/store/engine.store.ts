// Responsibility: engine process lifecycle — run/stop plus the second-confirmation state machine for terminating.

import { createStore } from "solid-js/store";

import { getEngineStatus, startEngine, stopEngine } from "@/ipc";
import { logger } from "@/utils";

const log = logger.child("engine");

interface EngineState {
  running: boolean;
  pending: boolean; // "Stop" already clicked once, awaiting second confirmation
  pendingTimer: ReturnType<typeof setTimeout> | undefined;
}

const [engineStore, setEngineStore] = createStore<EngineState>({
  running: false,
  pending: false,
  pendingTimer: undefined,
});

/** In sync with the real daemon state */
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
      // Second confirmation → actually terminate
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
      // First click → enter the pending state; auto-abandon if it is not confirmed within 2.2s
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
