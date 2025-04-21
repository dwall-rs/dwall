import { createSignal } from "solid-js";

type TimeoutID = ReturnType<typeof setTimeout>;

/**
 * A custom hook that provides pausable timeout functionality
 * @param callback The function to be executed after the timeout
 * @param delay The delay in milliseconds
 * @returns An array containing the start, pause, resume, and clear functions
 */
export function usePausableTimeout(callback: () => void, delay: number) {
  const [remainingTime, setRemainingTime] = createSignal<number | null>(null);
  const [timeoutId, setTimeoutId] = createSignal<TimeoutID | null>(null);
  const [startTime, setStartTime] = createSignal<number | null>(null);
  const [isPaused, setIsPaused] = createSignal(false);

  const clear = () => {
    if (timeoutId()) {
      clearTimeout(timeoutId()!);
      setTimeoutId(null);
    }
    setRemainingTime(null);
    setStartTime(null);
    setIsPaused(false);
  };

  const start = () => {
    clear();
    setRemainingTime(delay);
    setStartTime(performance.now());
    setIsPaused(false);
    const id = setTimeout(() => {
      callback();
      clear();
    }, delay);
    setTimeoutId(id);
  };

  const pause = () => {
    if (!timeoutId() || isPaused()) return;

    clearTimeout(timeoutId()!);
    setTimeoutId(null);
    setIsPaused(true);

    const elapsedTime = performance.now() - (startTime() || 0);
    setRemainingTime((prev) => Math.max((prev || 0) - elapsedTime, 0));
  };

  const resume = () => {
    if (!isPaused() || !remainingTime()) return;

    setStartTime(performance.now());
    setIsPaused(false);
    const remaining = remainingTime();
    if (!remaining || remaining <= 0) {
      callback();
      return clear();
    }

    const id = setTimeout(() => {
      callback();
      clear();
    }, remaining);
    setTimeoutId(id);
  };

  return { start, pause, resume, clear } as const;
}
