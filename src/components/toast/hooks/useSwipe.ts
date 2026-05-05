import { createSignal } from "solid-js";
import { SWIPE_THRESHOLD } from "../constants";
import { getDefaultSwipeDirections } from "../utils";
import type { SwipeDirection, ToastT } from "../types";

interface UseSwipeOptions {
  toastRef: () => HTMLLIElement | undefined;
  toast: ToastT;
  dismissible: boolean;
  position: string;
  swipeDirections?: SwipeDirection[];
  offset: () => number;
  deleteToast: () => void;
}

export function useSwipe(opts: UseSwipeOptions) {
  const [swiping, setSwiping] = createSignal(false);
  const [swipeOut, setSwipeOut] = createSignal(false);
  const [swipeOutDirection, setSwipeOutDirection] = createSignal<
    "left" | "right" | "up" | "down" | null
  >(null);
  const [isSwiped, setIsSwiped] = createSignal(false);
  const [offsetBeforeRemove, setOffsetBeforeRemove] = createSignal(0);

  let swipeDirection: "x" | "y" | null = null;
  let pointerStart: { x: number; y: number } | null = null;
  let dragStartTime: Date | null = null;

  const disabled = () => opts.toast.type === "loading";

  const onPointerDown = (e: PointerEvent) => {
    if (e.button === 2) return;
    if (disabled() || !opts.dismissible) return;
    dragStartTime = new Date();
    setOffsetBeforeRemove(opts.offset());
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    if ((e.target as HTMLElement).tagName === "BUTTON") return;
    setSwiping(true);
    pointerStart = { x: e.clientX, y: e.clientY };
  };

  const onPointerUp = () => {
    if (swipeOut() || !opts.dismissible) return;
    pointerStart = null;

    const el = opts.toastRef();
    const amtX = Number(
      el?.style.getPropertyValue("--swipe-amount-x").replace("px", "") || 0,
    );
    const amtY = Number(
      el?.style.getPropertyValue("--swipe-amount-y").replace("px", "") || 0,
    );
    const timeTaken = Date.now() - (dragStartTime?.getTime() ?? 0);
    const swipeAmt = swipeDirection === "x" ? amtX : amtY;
    const velocity = Math.abs(swipeAmt) / timeTaken;

    if (Math.abs(swipeAmt) >= SWIPE_THRESHOLD || velocity > 0.11) {
      setOffsetBeforeRemove(opts.offset());
      opts.toast.onDismiss?.(opts.toast);
      if (swipeDirection === "x") {
        setSwipeOutDirection(amtX > 0 ? "right" : "left");
      } else {
        setSwipeOutDirection(amtY > 0 ? "down" : "up");
      }
      opts.deleteToast();
      setSwipeOut(true);
      return;
    }

    el?.style.setProperty("--swipe-amount-x", "0px");
    el?.style.setProperty("--swipe-amount-y", "0px");
    setIsSwiped(false);
    setSwiping(false);
    swipeDirection = null;
  };

  const onPointerMove = (e: PointerEvent) => {
    if (!pointerStart || !opts.dismissible) return;
    if ((window.getSelection()?.toString().length ?? 0) > 0) return;

    const yDelta = e.clientY - pointerStart.y;
    const xDelta = e.clientX - pointerStart.x;
    const dirs =
      opts.swipeDirections ?? getDefaultSwipeDirections(opts.position);

    if (!swipeDirection && (Math.abs(xDelta) > 1 || Math.abs(yDelta) > 1)) {
      swipeDirection = Math.abs(xDelta) > Math.abs(yDelta) ? "x" : "y";
    }

    const getDampen = (delta: number) => 1 / (1.5 + Math.abs(delta) / 20);

    const swipeAmt = { x: 0, y: 0 };

    if (
      swipeDirection === "y" &&
      (dirs.includes("top") || dirs.includes("bottom"))
    ) {
      if (
        (dirs.includes("top") && yDelta < 0) ||
        (dirs.includes("bottom") && yDelta > 0)
      ) {
        swipeAmt.y = yDelta;
      } else {
        const d = yDelta * getDampen(yDelta);
        swipeAmt.y = Math.abs(d) < Math.abs(yDelta) ? d : yDelta;
      }
    } else if (
      swipeDirection === "x" &&
      (dirs.includes("left") || dirs.includes("right"))
    ) {
      if (
        (dirs.includes("left") && xDelta < 0) ||
        (dirs.includes("right") && xDelta > 0)
      ) {
        swipeAmt.x = xDelta;
      } else {
        const d = xDelta * getDampen(xDelta);
        swipeAmt.x = Math.abs(d) < Math.abs(xDelta) ? d : xDelta;
      }
    }

    if (Math.abs(swipeAmt.x) > 0 || Math.abs(swipeAmt.y) > 0) setIsSwiped(true);

    const el = opts.toastRef();
    el?.style.setProperty("--swipe-amount-x", `${swipeAmt.x}px`);
    el?.style.setProperty("--swipe-amount-y", `${swipeAmt.y}px`);
  };

  const onDragEnd = () => {
    setSwiping(false);
    swipeDirection = null;
    pointerStart = null;
  };

  return {
    swiping,
    swipeOut,
    swipeOutDirection,
    isSwiped,
    offsetBeforeRemove,
    setOffsetBeforeRemove,
    onPointerDown,
    onPointerUp,
    onPointerMove,
    onDragEnd,
  };
}
