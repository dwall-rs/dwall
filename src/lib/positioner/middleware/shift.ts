import type { Boundary, Middleware } from "../types";
import { getViewportBoundary } from "../utils/dom";

export interface ShiftOptions {
  /** Minimum gap from the boundary, default 0 */
  padding?: number;
  /** Custom boundary rect; defaults to the viewport */
  boundary?: Boundary;
  /** Whether to constrain the main axis (e.g. the y direction of top/bottom placements), default true */
  mainAxis?: boolean;
  /** Whether to constrain the cross axis (e.g. the x direction of top/bottom placements), default true */
  crossAxis?: boolean;
}

/** Translate the floating element to stay within the boundary (viewport by default), without flipping the placement */
export function shift(options: ShiftOptions = {}): Middleware {
  return {
    name: "shift",
    fn(state) {
      const { x, y, rects, strategy } = state;
      const { padding = 0, mainAxis = true, crossAxis = true } = options;
      const boundary =
        options.boundary ?? getViewportBoundary(strategy, padding);

      let nextX = x;
      let nextY = y;

      if (crossAxis) {
        const minX = boundary.x;
        const maxX = boundary.x + boundary.width - rects.floating.width;
        nextX = Math.min(Math.max(x, minX), Math.max(minX, maxX));
      }

      if (mainAxis) {
        const minY = boundary.y;
        const maxY = boundary.y + boundary.height - rects.floating.height;
        nextY = Math.min(Math.max(y, minY), Math.max(minY, maxY));
      }

      return {
        x: nextX,
        y: nextY,
        data: { x: nextX - x, y: nextY - y },
      };
    },
  };
}
