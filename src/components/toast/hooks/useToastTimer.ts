/**
 * hooks/useToastTimer.ts
 * Manages the auto-dismiss countdown for a single toast.
 * Single responsibility: start, pause, and resume a timer; call deleteToast when done.
 */
import { createEffect, onCleanup } from "solid-js";
import type { ToastT } from "../types";

interface UseToastTimerOptions {
  toast: ToastT;
  duration: number;
  expanded: () => boolean;
  interacting: () => boolean;
  isDocumentHidden: () => boolean;
  deleteToast: () => void;
}

export function useToastTimer(opts: UseToastTimerOptions) {
  let remainingTime = opts.toast.duration ?? opts.duration;
  let closeTimerStart = 0;
  let lastPauseTime = 0;
  let timeoutId: ReturnType<typeof setTimeout> | undefined;

  const shouldSkip = () =>
    (opts.toast.promise && opts.toast.type === "loading") ||
    opts.toast.duration === Infinity ||
    opts.toast.type === "loading";

  const pauseTimer = () => {
    if (lastPauseTime < closeTimerStart) {
      const elapsed = Date.now() - closeTimerStart;
      remainingTime = Math.max(0, remainingTime - elapsed);
    }
    lastPauseTime = Date.now();
  };

  const startTimer = () => {
    if (remainingTime === Infinity) return;
    clearTimeout(timeoutId);
    closeTimerStart = Date.now();
    timeoutId = setTimeout(() => {
      opts.toast.onAutoClose?.(opts.toast);
      opts.deleteToast();
    }, remainingTime);
  };

  createEffect(() => {
    if (shouldSkip()) return;
    if (opts.expanded() || opts.interacting() || opts.isDocumentHidden()) {
      pauseTimer();
    } else {
      startTimer();
    }
    onCleanup(() => clearTimeout(timeoutId));
  });
}
