import type { ReferenceElement, Rect, Strategy } from "../types";

/** Convert a DOMRect to a plain data object (viewport coordinate system); also handles virtual reference elements */
export function getViewportRect(el: ReferenceElement): Rect {
  const rect = el.getBoundingClientRect();
  return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
}

/**
 * Get the rect participating in the computation for a strategy:
 * - fixed: use viewport coordinates directly (simplest and most reliable with position:fixed)
 * - absolute: add the page scroll offset to convert to "document coordinates"
 *   (assuming the floating element ends up mounted under <body>, i.e. the
 *    nearest positioned ancestor is body/html — the common Portal-to-body
 *    usage; if your floating element has a custom non-static positioned
 *    ancestor, prefer strategy: 'fixed')
 */
export function getRectRelativeTo(
  el: ReferenceElement,
  strategy: Strategy,
): Rect {
  const rect = getViewportRect(el);
  if (strategy === "fixed") return rect;
  const scrollX = window.scrollX ?? window.pageXOffset;
  const scrollY = window.scrollY ?? window.pageYOffset;
  return {
    x: rect.x + scrollX,
    y: rect.y + scrollY,
    width: rect.width,
    height: rect.height,
  };
}

/** The current viewport as the default boundary (for shift / flip overflow detection); strategy decides the coordinate system */
export function getViewportBoundary(strategy: Strategy, padding = 0): Rect {
  const width = document.documentElement.clientWidth;
  const height = document.documentElement.clientHeight;
  const scrollX =
    strategy === "fixed" ? 0 : (window.scrollX ?? window.pageXOffset);
  const scrollY =
    strategy === "fixed" ? 0 : (window.scrollY ?? window.pageYOffset);
  return {
    x: scrollX + padding,
    y: scrollY + padding,
    width: width - padding * 2,
    height: height - padding * 2,
  };
}

function isOverflowElement(el: Element): boolean {
  const { overflow, overflowX, overflowY } = getComputedStyle(el);
  return /auto|scroll|overlay|hidden/.test(overflow + overflowX + overflowY);
}

/** Find all scrollable ancestors that may affect layout + window, for autoUpdate to listen to scroll/resize */
export function getOverflowAncestors(node: Element): Array<Element | Window> {
  const result: Array<Element | Window> = [];
  let el: Element | null = node.parentElement;

  while (el) {
    if (isOverflowElement(el)) {
      result.push(el);
    }
    el = el.parentElement;
  }

  result.push(window);
  return result;
}
