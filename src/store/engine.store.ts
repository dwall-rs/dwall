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
import { queueConfigWrite } from "./config-write-queue";
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
/** The same, for the log read: it is a different question from the process status. */
let lastErrorReadFailed = false;
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

/** Reads the daemon's last error, reporting a failure without blaming the status query. */
const refreshLastError = async (seq: number) => {
  try {
    await readLastError(seq);
    lastErrorReadFailed = false;
  } catch (e) {
    if (seq !== refreshSeq) return;
    if (!lastErrorReadFailed) notifyFailure("read the daemon's last error", e);
    lastErrorReadFailed = true;
  }
};

/** In sync with the real daemon state */
const refresh = async () => {
  const seq = ++refreshSeq;

  // The status and the log are separate questions: a log read that fails must not make
  // the window say the engine query failed, nor hold back the status it already has.
  let running: boolean;
  try {
    running = await getEngineStatus();
  } catch (e) {
    if (seq !== refreshSeq) return;
    // The poll runs every few seconds; one toast per failure streak is a report, one
    // per tick is spam.
    if (!refreshFailed) notifyFailure("query engine status", e);
    refreshFailed = true;
    return;
  }

  // A newer refresh started while this one was in flight; its result is the current one.
  if (seq !== refreshSeq) return;
  refreshFailed = false;
  setEngineStore("running", running);

  // A stopped daemon used to drop its last error, which is exactly when the reason
  // matters: the daemon stops for good when the user's own setting leaves it without
  // a location, and the log line it wrote is the only explanation there is.
  if (running) {
    stoppedErrorRead = false;
    await refreshLastError(seq);
  } else if (!stoppedErrorRead) {
    stoppedErrorRead = true;
    await refreshLastError(seq);
  }
};

/** Terminates the daemon and mirrors the cleared mode into the settings draft. */
const stopNow = async () => {
  // No running check here: this task runs only after the writes queued before it, and
  // by then the daemon may have exited on its own or been restarted by an apply. The
  // Rust command clears wallpaper_mode before stopping and is idempotent when no
  // process is left, so the stop has to go through even with a stale running flag;
  // returning early would leave the persisted mode and the next window open would
  // start the daemon again.
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
 * Stops the engine after every earlier configuration write has finished.
 *
 * The command clears `wallpaper_mode` in `config.toml`. If a save that was already
 * in flight wrote after it, the cleared mode came back and the next window open
 * started the daemon again; the shared queue makes the stop the last write. Because
 * the task is queued, the running flag is deliberately not checked when it runs:
 * a poll can observe the daemon stopped during the wait, and the stop still has to
 * clear the configuration.
 */
const stop = () => queueConfigWrite(stopNow);

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
