import type { Middleware } from "../types";
import { getSide, isVerticalSide } from "../core/placement";

export interface ArrowOptions {
  /**
   * The arrow DOM element, or a function returning it (in Solid, usually an
   * accessor). The type is Element rather than HTMLElement to support arrows
   * drawn with <svg> in SVG scenarios (SVGSVGElement is not an HTMLElement
   * subtype).
   */
  element: Element | undefined | (() => Element | undefined);
  /** Minimum gap between the arrow and the floating's edge, default 4 */
  padding?: number;
}

export interface ArrowData {
  x?: number;
  y?: number;
  /** Which side of the floating the arrow should hug (the placement's main side of the floating) */
  side: ReturnType<typeof getSide>;
}

/**
 * Compute the arrow's x/y offset relative to the floating element's top-left,
 * so it always points at the reference element's center while staying within
 * the floating's bounds.
 */
export function arrow(options: ArrowOptions): Middleware {
  return {
    name: "arrow",
    fn(state) {
      const el =
        typeof options.element === "function"
          ? options.element()
          : options.element;
      const side = getSide(state.placement);
      if (!el) return { data: { side } };

      const { rects, x, y } = state;
      const padding = options.padding ?? 4;
      const vertical = isVerticalSide(side);

      // Use getBoundingClientRect rather than offsetWidth/offsetHeight: the
      // latter exist only on HTMLElement; SVG elements (e.g. <svg>/<g>) have
      // neither. getBoundingClientRect works for any Element as long as the
      // arrow has no CSS transform (TooltipArrow avoids rotate for this).
      const arrowRect = el.getBoundingClientRect();
      const arrowWidth = arrowRect.width || 0;
      const arrowHeight = arrowRect.height || 0;

      // Key: read the floating's size fresh instead of using
      // state.rects.floating (cached at the start of this computePosition
      // pass). If a middleware like size, ahead of arrow, produces DOM side
      // effects via its apply callback (e.g. reactively changing max-width)
      // and they land synchronously, state.rects.floating holds the old
      // pre-change size, so the arrow offset computed from it mismatches the
      // finally rendered box width. Re-measuring always gets the current size.
      const freshFloatingRect = state.elements.floating.getBoundingClientRect();
      const axisLen = vertical
        ? freshFloatingRect.width
        : freshFloatingRect.height;
      const arrowLen = vertical ? arrowWidth : arrowHeight;

      // Position of the reference's center relative to the floating's top-left
      const refCenter = vertical
        ? rects.reference.x + rects.reference.width / 2 - x
        : rects.reference.y + rects.reference.height / 2 - y;

      let pos = refCenter - arrowLen / 2;
      const min = padding;
      const max = axisLen - arrowLen - padding;
      pos = Math.min(Math.max(pos, min), Math.max(min, max));

      const data: ArrowData = vertical ? { x: pos, side } : { y: pos, side };
      return { data };
    },
  };
}
