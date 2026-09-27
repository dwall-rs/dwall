import type { Boundary, Middleware, Placement, Side } from "../types";
import { getViewportBoundary } from "../utils/dom";
import {
  computeCoordsFromPlacement,
  getOppositePlacement,
  getSide,
} from "../core/placement";

export interface FlipOptions {
  padding?: number;
  boundary?: Boundary;
  /** Ordered list of candidate placements to try when flipping; by default only flips to the opposite side */
  fallbackPlacements?: Placement[];
}

interface SideOverflow {
  top: number;
  right: number;
  bottom: number;
  left: number;
}

function getSideOverflow(
  x: number,
  y: number,
  width: number,
  height: number,
  boundary: Boundary,
): SideOverflow {
  return {
    top: boundary.y - y,
    left: boundary.x - x,
    right: x + width - (boundary.x + boundary.width),
    bottom: y + height - (boundary.y + boundary.height),
  };
}

/** The overflow amount along a placement's main axis (positive = pixels overflowing) */
function getMainAxisOverflow(side: Side, overflow: SideOverflow): number {
  switch (side) {
    case "top":
      return overflow.top;
    case "bottom":
      return overflow.bottom;
    case "left":
      return overflow.left;
    case "right":
      return overflow.right;
  }
}

/** When the main axis has insufficient room, flip to the opposite placement (e.g. bottom -> top) */
export function flip(options: FlipOptions = {}): Middleware {
  return {
    name: "flip",
    fn(state) {
      const {
        x,
        y,
        rects,
        placement,
        strategy,
        middlewareData,
        initialPlacement,
      } = state;

      // Already flipped once; don't flip again to avoid oscillating back and forth
      if (middlewareData.flip?.flipped) return {};

      const { padding = 0 } = options;
      const boundary =
        options.boundary ?? getViewportBoundary(strategy, padding);

      const currentOverflow = getSideOverflow(
        x,
        y,
        rects.floating.width,
        rects.floating.height,
        boundary,
      );
      const currentMainOverflow = getMainAxisOverflow(
        getSide(placement),
        currentOverflow,
      );

      // The current direction fits; no flip needed
      if (currentMainOverflow <= 0) return {};

      const candidates = options.fallbackPlacements ?? [
        getOppositePlacement(initialPlacement),
      ];

      let best: { placement: Placement; overflow: number } | null = null;

      for (const candidate of candidates) {
        if (candidate === placement) continue;

        // The key fix: actually compute the candidate placement's coordinates
        // and its own real overflow, instead of wrongly mirroring the current
        // (already overflowing) placement's overflow.
        const candidateCoords = computeCoordsFromPlacement(rects, candidate);
        const candidateOverflow = getSideOverflow(
          candidateCoords.x,
          candidateCoords.y,
          rects.floating.width,
          rects.floating.height,
          boundary,
        );
        const candidateMainOverflow = getMainAxisOverflow(
          getSide(candidate),
          candidateOverflow,
        );

        if (candidateMainOverflow <= 0) {
          best = { placement: candidate, overflow: candidateMainOverflow };
          break; // Found a candidate that doesn't overflow at all; take it
        }
        if (!best || candidateMainOverflow < best.overflow) {
          best = { placement: candidate, overflow: candidateMainOverflow };
        }
      }

      // No candidate, or flipping isn't better than the current state: don't flip
      if (!best || best.overflow >= currentMainOverflow) return {};

      return {
        reset: { placement: best.placement },
        data: { flipped: true },
      };
    },
  };
}
