import type {
  ToggleGroupContextValue,
  ToggleGroupValue,
} from "./ToggleGroup.types";

/** Keyboard navigation keys: arrow keys + Home/End (aligned with base-ui CompositeRoot's enableHomeAndEndKeys) */
const NAV_KEYS = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

/**
 * Roving focus keyboard navigation for ToggleGroup (pure logic, unit-testable
 * on its own).
 *
 * Direction mapping:
 * - horizontal + ltr: ArrowRight → next, ArrowLeft → prev
 * - horizontal + rtl: left/right reversed
 * - vertical:          ArrowDown → next, ArrowUp → prev
 * - Home → first, End → last
 * - loopFocus=true wraps, false stops at the boundary; disabled items are
 *   skipped
 *
 * Keys are only taken over when focus is already on some item, to avoid
 * affecting other external elements; after moving focus the item's onFocus
 * updates the highlight.
 */
export function useToggleGroupKeyboard<
  TValue extends ToggleGroupValue = ToggleGroupValue,
>(ctx: ToggleGroupContextValue<TValue>) {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (ctx.disabled()) return;
    if (!NAV_KEYS.has(e.key)) return;

    const activeEl = document.activeElement as HTMLElement | null;
    const items = ctx.getItems().filter((item) => !item.disabled());
    if (items.length === 0) return;

    const currentIndex = items.findIndex((item) => item.element === activeEl);
    if (currentIndex === -1) return;

    const isVertical = ctx.orientation() === "vertical";
    // When dir is unspecified/auto, use the root element's actual writing direction (honoring dir on ancestors)
    const dir = ctx.dir();
    const isRTL =
      dir === "rtl" ||
      (dir !== "ltr" &&
        typeof document !== "undefined" &&
        getComputedStyle(e.currentTarget as Element).direction === "rtl");

    let nextIndex: number | undefined;
    switch (e.key) {
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = items.length - 1;
        break;
      case "ArrowLeft":
        if (isVertical) return;
        nextIndex = currentIndex + (isRTL ? 1 : -1);
        break;
      case "ArrowRight":
        if (isVertical) return;
        nextIndex = currentIndex + (isRTL ? -1 : 1);
        break;
      case "ArrowUp":
        if (!isVertical) return;
        nextIndex = currentIndex - 1;
        break;
      case "ArrowDown":
        if (!isVertical) return;
        nextIndex = currentIndex + 1;
        break;
    }

    if (nextIndex === undefined) return;

    // The arrow key matches the current layout; take over the default scroll behavior
    e.preventDefault();

    if (ctx.loopFocus()) {
      nextIndex = (nextIndex + items.length) % items.length;
    } else {
      nextIndex = Math.min(items.length - 1, Math.max(0, nextIndex));
    }

    items[nextIndex].element.focus();
    ctx.setHighlightedValue(items[nextIndex].value);
  };

  return { handleKeyDown };
}
