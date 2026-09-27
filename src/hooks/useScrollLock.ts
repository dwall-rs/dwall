import { createEffect, onCleanup, type Accessor } from "solid-js";

export interface UseScrollLockOptions {
  /**
   * Selector of a container that may keep scrolling while the lock is held (usually the overlay itself).
   * Wheel/touch scrolling that misses this selector is blocked; omit it to lock scrolling for the whole page.
   */
  allowedSelector?: string;
}

// Multiple overlays/nested overlays may hold the lock at once; a counter keeps one closing from restoring page scrolling early.
let lockCount = 0;
let release: (() => void) | undefined;
// Selectors of the containers each locked overlay allows scrolling in. preventScroll reads this live set,
// so a selector added after the lock takes effect is honored immediately.
const allowedSelectors = new Set<string>();

/** Whether the element itself establishes a scroll container (same criterion as in the positioner) */
function isOverflowElement(el: Element): boolean {
  const { overflow, overflowX, overflowY } = getComputedStyle(el);
  return /auto|scroll|overlay|hidden/.test(overflow + overflowX + overflowY);
}

/**
 * The page's scroll container: lock html when html itself establishes a scroll context, otherwise lock body
 * — putting overflow on some other element does not lock the page (Base UI makes the same check).
 */
function getViewportScroller(): HTMLElement {
  const html = document.documentElement;
  return isOverflowElement(html) ? html : document.body;
}

function supportsStableScrollbarGutter(): boolean {
  return (
    typeof CSS !== "undefined" &&
    typeof CSS.supports === "function" &&
    CSS.supports("scrollbar-gutter", "stable")
  );
}

function isWithinAllowedScroll(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  for (const selector of allowedSelectors) {
    if (target.closest(selector)) return true;
  }
  return false;
}

function lockViewport(): () => void {
  const html = document.documentElement;
  const body = document.body;
  const scroller = getViewportScroller();

  const previous = {
    htmlScrollbarGutter: html.style.scrollbarGutter,
    scrollerOverflowX: scroller.style.overflowX,
    scrollerOverflowY: scroller.style.overflowY,
    bodyPaddingRight: body.style.paddingRight,
    scrollTop: scroller.scrollTop,
    scrollLeft: scroller.scrollLeft,
  };

  // Offset the horizontal jitter caused by the scrollbar disappearing: use scrollbar-gutter when available,
  // otherwise fall back to padding body's right side by the scrollbar width.
  // Compensate only when the page itself really has a placeholder scrollbar: when the scrollbar belongs to a
  // nested container, locking html/body does not remove it, and adding a gutter to html would only insert
  // extra blank space (scrollbar + gutter = double width), breaking the layout instead.
  const scrollbarWidth = Math.max(0, window.innerWidth - html.clientWidth);
  const gutterSupported = supportsStableScrollbarGutter();
  if (scrollbarWidth > 0) {
    if (gutterSupported) {
      html.style.scrollbarGutter = "stable";
    } else {
      const padding =
        Number.parseFloat(getComputedStyle(body).paddingRight) || 0;
      body.style.paddingRight = `${padding + scrollbarWidth}px`;
    }
  }

  scroller.style.overflowX = "hidden";
  scroller.style.overflowY = "hidden";

  // Fallback: some browsers still allow wheel/touch scrolling despite overflow:hidden,
  // so scroll events outside the overlay are intercepted; inside (matching allowedSelector) scrolling stays possible.
  const preventScroll = (event: Event) => {
    if (isWithinAllowedScroll(event.target)) return;
    event.preventDefault();
  };
  document.addEventListener("wheel", preventScroll, { passive: false });
  document.addEventListener("touchmove", preventScroll, { passive: false });

  return () => {
    html.style.scrollbarGutter = previous.htmlScrollbarGutter;
    if (!gutterSupported) {
      body.style.paddingRight = previous.bodyPaddingRight;
    }
    scroller.style.overflowX = previous.scrollerOverflowX;
    scroller.style.overflowY = previous.scrollerOverflowY;
    document.removeEventListener("wheel", preventScroll);
    document.removeEventListener("touchmove", preventScroll);
    scroller.scrollTop = previous.scrollTop;
    scroller.scrollLeft = previous.scrollLeft;
  };
}

function acquireLock(allowedSelector?: string): () => void {
  lockCount += 1;
  if (allowedSelector) allowedSelectors.add(allowedSelector);
  if (lockCount === 1) release = lockViewport();

  let released = false;
  return () => {
    if (released) return;
    released = true;
    lockCount -= 1;
    if (lockCount === 0) {
      release?.();
      release = undefined;
      // Clear the allowlist once everything is released so leftover selectors do not affect the next lock.
      allowedSelectors.clear();
    }
  };
}

/**
 * Lock page scrolling while an overlay is open and restore it on close — matching Base UI's modal behavior
 * (document scroll locked, wheel/touch scrolling outside the overlay intercepted), so floating anchors do not
 * "drift" along with the page.
 *
 * Multiple overlays may hold the lock at once (reference counted); containers matching `allowedSelector`
 * remain scrollable, usually the overlay itself — e.g. Popover passes `[data-slot="popover-content"]`.
 */
export function useScrollLock(
  locked: Accessor<boolean>,
  options?: UseScrollLockOptions,
) {
  createEffect(() => {
    if (!locked()) return;
    const unlock = acquireLock(options?.allowedSelector);
    onCleanup(unlock);
  });
}
