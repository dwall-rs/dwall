// Responsibility: a countdown that can be held and resumed, for UI that must not
//       disappear while it is being read.

/**
 * A timeout that can be paused and resumed.
 *
 * A plain `setTimeout` cannot do this: clearing it throws away how much of the wait
 * was left. This keeps the deadline (while running) or the remainder (while held), so
 * a toast can stop under the cursor and finish the rest of its lifetime afterwards
 * instead of starting over.
 */
export interface HoldableTimer {
  /** Starts, or restarts, the countdown with `delay` milliseconds. */
  start: (delay: number) => void;
  /** Stops the countdown; `release` continues with what is left. */
  hold: () => void;
  /** Continues a held countdown, if there is any time left. */
  release: () => void;
  /** Cancels the countdown for good. */
  stop: () => void;
}

export function createHoldableTimer(expire: () => void): HoldableTimer {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let expiresAt = 0;
  let remaining = 0;
  let held = false;

  const clear = () => {
    if (timer === undefined) return;

    clearTimeout(timer);
    timer = undefined;
  };

  const schedule = () => {
    clear();
    if (held || remaining <= 0) return;

    expiresAt = Date.now() + remaining;
    timer = setTimeout(() => {
      timer = undefined;
      // Cleared, so a later `release` cannot re-arm a countdown that has run out.
      remaining = 0;
      expire();
    }, remaining);
  };

  return {
    start: (delay: number) => {
      remaining = delay;
      schedule();
    },
    hold: () => {
      held = true;
      if (timer === undefined) return;

      remaining = Math.max(0, expiresAt - Date.now());
      clear();
    },
    release: () => {
      held = false;
      schedule();
    },
    stop: () => {
      clear();
      remaining = 0;
    },
  };
}
