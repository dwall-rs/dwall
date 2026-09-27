import type { ReferenceElement } from "./types";
import { getOverflowAncestors } from "./utils/dom";

export interface AutoUpdateOptions {
  /** Trigger updates when an ancestor scroll container scrolls, default true */
  ancestorScroll?: boolean;
  /** Trigger updates when an ancestor container resizes (via ResizeObserver), default true */
  ancestorResize?: boolean;
  /** Trigger updates when the reference / floating itself changes size, default true (virtual reference elements have no size and are skipped automatically) */
  elementResize?: boolean;
}

/**
 * Automatically listens to all events that can affect positioning (scroll,
 * window/element size changes) and calls update on change; returns a cleanup
 * function that removes every listener.
 *
 * A virtual reference (e.g. a context menu anchored to mouse coordinates) has
 * no real DOM node for finding ancestor scroll containers or observing sizes:
 * - with `contextElement` provided, it is used to find scrollable ancestors;
 * - otherwise only window resize is watched (no scroll — no associated
 *   container to judge relevance by).
 */
export function autoUpdate(
  reference: ReferenceElement,
  floating: HTMLElement,
  update: () => void,
  options: AutoUpdateOptions = {},
): () => void {
  const {
    ancestorScroll = true,
    ancestorResize = true,
    elementResize = true,
  } = options;

  const isRealElement = reference instanceof Element;
  const ancestorAnchor: Element | null = isRealElement
    ? reference
    : ((reference as { contextElement?: Element }).contextElement ?? null);

  const ancestors =
    (ancestorScroll || ancestorResize) && ancestorAnchor
      ? getOverflowAncestors(ancestorAnchor)
      : [];

  ancestors.forEach((ancestor) => {
    if (ancestorScroll) {
      ancestor.addEventListener("scroll", update, { passive: true });
    }
  });

  let resizeObserver: ResizeObserver | null = null;
  if (
    (ancestorResize || elementResize) &&
    typeof ResizeObserver !== "undefined"
  ) {
    resizeObserver = new ResizeObserver(() => update());
    // A virtual reference has no real size to observe; observe only the floating itself
    if (elementResize) {
      if (isRealElement) resizeObserver.observe(reference);
      resizeObserver.observe(floating);
    }
    if (ancestorResize) {
      ancestors.forEach((ancestor) => {
        if (ancestor instanceof Element) resizeObserver!.observe(ancestor);
      });
    }
  }

  window.addEventListener("resize", update);

  return () => {
    ancestors.forEach((ancestor) => {
      ancestor.removeEventListener("scroll", update);
    });
    resizeObserver?.disconnect();
    window.removeEventListener("resize", update);
  };
}
