import type { TabsContextValue } from "./Tabs.context";

const NAV_KEYS = new Set([
  "ArrowLeft",
  "ArrowRight",
  "ArrowUp",
  "ArrowDown",
  "Home",
  "End",
]);

/**
 * Roving focus keyboard navigation for the tablist (pure logic, unit-testable
 * independently).
 *
 * Direction mapping:
 * - horizontal + ltr: ArrowRight → next, ArrowLeft → prev
 * - horizontal + rtl: left/right reversed
 * - vertical:          ArrowDown → next, ArrowUp → prev
 * - Home → first, End → last
 * - loop=true wraps, false stops at the boundary; skips disabled triggers
 *
 * Matches base-ui: arrow keys only move focus (onFocus updates the highlight),
 * never activate; activation happens on click (Enter/Space triggers the
 * button's click). Focus activation under activateOnFocus is handled by
 * TabsTrigger's onFocus, not here.
 */
export function useTabsKeyboard(ctx: TabsContextValue) {
  const handleKeyDown = (e: KeyboardEvent) => {
    if (!NAV_KEYS.has(e.key)) return;

    const activeEl = document.activeElement as HTMLElement | null;
    // Only take over the keyboard when focus is on a tab
    if (activeEl?.getAttribute("role") !== "tab") return;

    const tabs = ctx.getTriggers().filter((t) => !t.disabled());
    if (tabs.length === 0) return;

    const currentIndex = tabs.findIndex((t) => t.element === activeEl);
    if (currentIndex === -1) return;

    const isVertical = ctx.orientation() === "vertical";
    // Fall back to the document's actual direction when dir=auto
    const dir = ctx.dir();
    const isRTL =
      dir === "rtl" ||
      (dir === "auto" &&
        typeof document !== "undefined" &&
        document.documentElement.dir === "rtl");

    let nextIndex: number | undefined;
    switch (e.key) {
      case "Home":
        nextIndex = 0;
        break;
      case "End":
        nextIndex = tabs.length - 1;
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

    // The arrow key matches the current layout; take over default scrolling
    e.preventDefault();

    if (ctx.loop()) {
      nextIndex = (nextIndex + tabs.length) % tabs.length;
    } else {
      nextIndex = Math.min(tabs.length - 1, Math.max(0, nextIndex));
    }

    // Focusing triggers the trigger's onFocus → update highlight (and activate when activateOnFocus)
    tabs[nextIndex].element.focus();
  };

  return { handleKeyDown };
}
