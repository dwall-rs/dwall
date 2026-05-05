import { onCleanup, onMount } from "solid-js";

export const useAutoPlay = (
  enabled: boolean,
  interval: number,
  scrollNext: () => void,
) => {
  if (!enabled) return;

  let timer: ReturnType<typeof setInterval>;

  const start = () => {
    timer = setInterval(scrollNext, interval);
  };
  const stop = () => clearInterval(timer);

  onMount(start);
  onCleanup(stop);

  return { start, stop };
};
