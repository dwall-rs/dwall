// Responsibility: engine process lifecycle — run/stop plus the second-confirmation state machine for terminating.

import { createStore } from "solid-js/store";

import {
  getDaemonLastError,
  getEngineStatus,
  startEngine,
  stopEngine,
} from "@/ipc";
import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { syncStoppedSettings } from "./settings-refresh";
import { logger } from "@/utils";

const log = logger.child("engine");

const notifyFailure = (action: string, error: unknown) => {
  log.error(`Failed to ${action}`, error);
  // Production builds compile the dev logger away, so failures must also reach
  // the user through the UI.
  toast.error(t("common.operationFailed"), { description: String(error) });
};

interface EngineState {
  running: boolean;
  /**
   * The daemon's last error line, or null when it has none.
   *
   * The daemon keeps running after a failed cycle, so "running" alone no longer
   * says whether the wallpapers are actually being applied.
   */
  lastError: string | null;
  pending: boolean; // "Stop" already clicked once, awaiting second confirmation
  pendingTimer: ReturnType<typeof setTimeout> | undefined;
}

const [engineStore, setEngineStore] = createStore<EngineState>({
  running: false,
  lastError: null,
  pending: false,
  pendingTimer: undefined,
});

/** Set while a refresh is failing, so the 5 s poll reports the outage once per streak. */
let refreshFailed = false;
/** Latest refresh issued; an older, slower one must not write its result over a newer one. */
let refreshSeq = 0;
/**
 * Whether the stopped daemon's log has already been read since it stopped.
 *
 * While running, the error line is re-read on every poll: a failed cycle is retried,
 * so the line can appear and then clear. While stopped it cannot change — the daemon
 * writes nothing — so it is read once, including on the first poll after a window
 * opens onto a daemon that has already exited.
 */
let stoppedErrorRead = false;

/** Reads the daemon's newest error line, unless a newer refresh has overtaken this one. */
const readLastError = async (seq: number) => {
  const lastError = await getDaemonLastError();
  if (seq !== refreshSeq) return;
  setEngineStore("lastError", lastError);
};

/** In sync with the real daemon state */
const refresh = async () => {
  const seq = ++refreshSeq;
  try {
    const running = await getEngineStatus();
    // A newer refresh started while this one was in flight; its result is the current one.
    if (seq !== refreshSeq) return;
    refreshFailed = false;
    setEngineStore("running", running);

    // A stopped daemon used to drop its last error, which is exactly when the reason
    // matters: the daemon stops for good when the user's own setting leaves it without
    // a location, and the log line it wrote is the only explanation there is.
    if (running) {
      stoppedErrorRead = false;
      await readLastError(seq);
    } else if (!stoppedErrorRead) {
      stoppedErrorRead = true;
      await readLastError(seq);
    }
  } catch (e) {
    if (seq !== refreshSeq) return;
    // The poll runs every few seconds; one toast per failure streak is a report, one
    // per tick is spam.
    if (!refreshFailed) notifyFailure("query engine status", e);
    refreshFailed = true;
  }
};

/** Terminates the daemon and mirrors the cleared mode into the settings draft. */
const stop = async () => {
  if (!engineStore.running) return;

  if (engineStore.pendingTimer) clearTimeout(engineStore.pendingTimer);

  try {
    await stopEngine();
    // The command cleared the theme in config.toml. Mirror only that: reloading
    // the file here also discarded every unsaved edit in the settings page.
    syncStoppedSettings();
    // A deliberate stop is not a failure: there is no reason to show, and the log
    // still holds whatever the last cycle wrote before it.
    stoppedErrorRead = true;
    setEngineStore({
      running: false,
      lastError: null,
      pending: false,
      pendingTimer: undefined,
    });
  } catch (e) {
    notifyFailure("stop engine", e);
    // The daemon may still be running: let the next poll read why.
    stoppedErrorRead = false;
    setEngineStore((prev) => ({
      ...prev,
      pending: false,
      pendingTimer: undefined,
    }));
  }
};

/**
 * Kept for keyboard/legacy callers: the UI now confirms with a dialog and calls
 * `stop` directly, but the two-step form remains the store's contract.
 */
const toggle = async () => {
  if (engineStore.running) {
    if (engineStore.pending) {
      await stop();
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
    // The launch command only spawns the process, so a daemon that exits immediately
    // reports nothing; its own log is the only place the reason appears. Read it on
    // the next poll.
    stoppedErrorRead = false;
    try {
      await startEngine();
      setEngineStore("running", true);
    } catch (e) {
      notifyFailure("start engine", e);
    }
  }
};

export { engineStore, toggle, stop, refresh };
