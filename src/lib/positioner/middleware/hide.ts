import type { Boundary, Middleware } from "../types";
import { getViewportBoundary, getRectRelativeTo } from "../utils/dom";

export interface HideOptions {
  padding?: number;
  boundary?: Boundary;
  /**
   * 'referenceHidden': the reference element is fully clipped away by its own scroll container (e.g. scrolled out of a scrollable parent)
   * 'escaped': the floating element is still within the viewport but has fully left the boundary the reference belongs to
   * (e.g. the reference is half blocked by horizontal scrolling inside a card; the floating can still be placed in the viewport,
   *  but should no longer be shown, otherwise it looks like it is "floating" over other content)
   * By default only referenceHidden is detected — the most common and most needed scenario.
   */
  strategy?: "referenceHidden" | "escaped";
}

export interface HideData {
  referenceHidden?: boolean;
  escaped?: boolean;
}

/**
 * Doesn't change coordinates; only produces "should the floating be hidden"
 * data for the caller to decide rendering: read
 * `middlewareData().hide?.referenceHidden` and close it when true.
 */
export function hide(options: HideOptions = {}): Middleware {
  return {
    name: "hide",
    fn(state) {
      const { x, y, rects, strategy: positionStrategy, elements } = state;
      const { padding = 0, strategy = "referenceHidden" } = options;
      const boundary =
        options.boundary ?? getViewportBoundary(positionStrategy, padding);

      const data: HideData = {};

      if (strategy === "referenceHidden") {
        const refRect = getRectRelativeTo(elements.reference, positionStrategy);
        const referenceHidden =
          refRect.x + refRect.width <= boundary.x ||
          refRect.y + refRect.height <= boundary.y ||
          refRect.x >= boundary.x + boundary.width ||
          refRect.y >= boundary.y + boundary.height;
        data.referenceHidden = referenceHidden;
      }

      if (strategy === "escaped") {
        const escaped =
          x + rects.floating.width <= boundary.x ||
          y + rects.floating.height <= boundary.y ||
          x >= boundary.x + boundary.width ||
          y >= boundary.y + boundary.height;
        data.escaped = escaped;
      }

      return { data };
    },
  };
}
