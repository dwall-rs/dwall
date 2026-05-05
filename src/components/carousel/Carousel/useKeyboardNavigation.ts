import { type Accessor, createEffect, onCleanup } from "solid-js";
import type { Orientation } from "./Carousel.types";

export function useKeyboardNavigation(
  ref: HTMLElement | undefined,
  orientation: Accessor<Orientation>,
  scrollPrev: () => void,
  scrollNext: () => void,
) {
  const handleKeyDown = (e: KeyboardEvent) => {
    const isH = orientation() === "horizontal";
    if ((isH && e.key === "ArrowLeft") || (!isH && e.key === "ArrowUp")) {
      e.preventDefault();
      scrollPrev();
    } else if (
      (isH && e.key === "ArrowRight") ||
      (!isH && e.key === "ArrowDown")
    ) {
      e.preventDefault();
      scrollNext();
    } else if (e.key === "Home") {
      e.preventDefault();
      // handled externally via scrollTo(0)
    } else if (e.key === "End") {
      e.preventDefault();
      // handled externally
    }
  };

  createEffect(() => {
    ref?.addEventListener("keydown", handleKeyDown);
    onCleanup(() => ref?.removeEventListener("keydown", handleKeyDown));
  });
}
