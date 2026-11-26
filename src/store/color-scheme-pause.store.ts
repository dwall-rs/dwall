// Responsibility: the colour-scheme pause shared by the engine section and the daemon.
//       The file is the source of truth: the daemon starts a pause by itself when it
//       notices a theme change it did not make, so the window polls instead of owning it.
import { createSignal } from "solid-js";

import { toast } from "@/components/ui/toast";
import { t } from "@/i18n";
import { getColorSchemePause, setColorSchemePause } from "@/ipc";
import type { ColorSchemePause } from "@/ipc";
import { logger } from "@/utils";

const log = logger.child("pause");

const [pause, setPause] = createSignal<ColorSchemePause | null>(null);

/**
 * Re-reads the pause from the backend.
 *
 * The engine section polls this every 30 s. A failed read must not reject — the caller
 * fires it with `void`, so a rejection becomes an unhandled promise rejection every
 * half minute — and it must keep the last known pause rather than show "not paused"
 * while the daemon may still be holding it.
 */
export const refreshColorSchemePause = async () => {
  try {
    setPause(await getColorSchemePause());
  } catch (error) {
    log.error("Failed to refresh the colour-scheme pause", error);
  }
};

/**
 * Pauses automatic switching for `minutes`, or resumes it when `minutes` is 0.
 *
 * Returns whether the backend accepted the change. A failure keeps the previous pause
 * and reports it, so the button is not silently inert.
 */
export const setColorSchemePauseFor = async (minutes: number) => {
  try {
    setPause(await setColorSchemePause(minutes));
    return true;
  } catch (error) {
    log.error("Failed to update the colour-scheme pause", error);
    toast.error(t("common.operationFailed"), { description: String(error) });
    return false;
  }
};

/**
 * Minutes left before switching resumes, or 0 when nothing is paused.
 *
 * Rounds up: a pause that ends in 20 seconds is still "1 minute" to the reader, and
 * showing 0 while it is still in force would suggest it has ended.
 */
export const colorSchemePauseRemainingMinutes = () => {
  const until = pause()?.until;
  if (!until) return 0;

  return Math.max(0, Math.ceil((until * 1000 - Date.now()) / 60_000));
};

export const colorSchemePauseStore = {
  pause,
  refreshColorSchemePause,
  setColorSchemePauseFor,
  remainingMinutes: colorSchemePauseRemainingMinutes,
};
