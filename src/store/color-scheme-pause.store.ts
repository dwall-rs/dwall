// Responsibility: the colour-scheme pause shared by the engine section and the daemon.
//       The file is the source of truth: the daemon starts a pause by itself when it
//       notices a theme change it did not make, so the window polls instead of owning it.
import { createSignal } from "solid-js";

import { getColorSchemePause, setColorSchemePause } from "@/ipc";
import type { ColorSchemePause } from "@/ipc";

const [pause, setPause] = createSignal<ColorSchemePause | null>(null);

/** Re-reads the pause from the backend. */
export const refreshColorSchemePause = async () => {
  setPause(await getColorSchemePause());
};

/** Pauses automatic switching for `minutes`, or resumes it when `minutes` is 0. */
export const setColorSchemePauseFor = async (minutes: number) => {
  setPause(await setColorSchemePause(minutes));
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
