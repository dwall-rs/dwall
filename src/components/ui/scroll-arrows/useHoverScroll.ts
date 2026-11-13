import { createEffect, onCleanup, type Accessor } from "solid-js";

export const HOVER_SCROLL_DELAY = 150;

export const HOVER_BAND_PX = 24;

export interface UseHoverScrollOptions {
  target: Accessor<HTMLElement | undefined>;

  canScrollUp: Accessor<boolean>;
  canScrollDown: Accessor<boolean>;

  enabled: Accessor<boolean>;
}

export function useHoverScroll(options: UseHoverScrollOptions): void {
  let frame: number | undefined;
  let timer: number | undefined;
  let direction: "up" | "down" | undefined;

  const stopLoop = () => {
    if (frame !== undefined) {
      cancelAnimationFrame(frame);
      frame = undefined;
    }
    direction = undefined;
  };

  const stop = () => {
    if (timer !== undefined) {
      clearTimeout(timer);
      timer = undefined;
    }
    stopLoop();
  };

  onCleanup(stop);

  createEffect(() => {
    const element = options.target();
    const enabled = options.enabled();
    if (!element || !enabled) return;

    const loop = () => {
      frame = undefined;

      const current = direction;
      if (current === undefined) return;

      const step = Math.max(4, element.clientHeight / 24);
      const before = element.scrollTop;
      element.scrollTop = before + (current === "down" ? step : -step);

      if (element.scrollTop === before) {
        stopLoop();
        return;
      }
      frame = requestAnimationFrame(loop);
    };

    const start = (next: "up" | "down") => {
      stopLoop();
      direction = next;
      frame = requestAnimationFrame(loop);
    };

    const schedule = (next: "up" | "down") => {
      if (direction === next) return;
      if (timer !== undefined) {
        clearTimeout(timer);
        timer = undefined;
      }
      stopLoop();
      timer = window.setTimeout(() => {
        timer = undefined;
        start(next);
      }, HOVER_SCROLL_DELAY);
    };

    const onPointerMove = (event: PointerEvent) => {
      const rect = element.getBoundingClientRect();
      const fromTop = event.clientY - rect.top;
      const fromBottom = rect.bottom - event.clientY;

      if (fromTop <= HOVER_BAND_PX && options.canScrollUp()) schedule("up");
      else if (fromBottom <= HOVER_BAND_PX && options.canScrollDown())
        schedule("down");
      else stop();
    };

    element.addEventListener("pointermove", onPointerMove);
    element.addEventListener("pointerleave", stop);

    element.addEventListener("pointerdown", stop);
    element.addEventListener("touchstart", stop, { passive: true });
    element.addEventListener("wheel", stop, { passive: true });
    element.addEventListener("keydown", stop);
    onCleanup(() => {
      stop();
      element.removeEventListener("pointermove", onPointerMove);
      element.removeEventListener("pointerleave", stop);
      element.removeEventListener("pointerdown", stop);
      element.removeEventListener("touchstart", stop);
      element.removeEventListener("wheel", stop);
      element.removeEventListener("keydown", stop);
    });
  });
}
