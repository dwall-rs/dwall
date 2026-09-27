import type { Middleware } from "../types";
import { getSide, isVerticalSide } from "../core/placement";

export interface OffsetValue {
  /** Offset along the main axis (away from/toward the reference), default 0 */
  mainAxis?: number;
  /** Offset along the cross axis (left/right or up/down translation), default 0 */
  crossAxis?: number;
}

export type OffsetOptions = number | OffsetValue;

/** Keep a gap between the floating and the reference, or fine-tune along the alignment axis */
export function offset(value: OffsetOptions = 0): Middleware {
  return {
    name: "offset",
    fn(state) {
      const { x, y, placement } = state;
      const side = getSide(placement);
      const vertical = isVerticalSide(side);

      const mainAxis =
        typeof value === "number" ? value : (value.mainAxis ?? 0);
      const crossAxis = typeof value === "number" ? 0 : (value.crossAxis ?? 0);

      // top/left are the "negative" direction, bottom/right the "positive" direction
      const mainSign = side === "bottom" || side === "right" ? 1 : -1;

      const diffMain = mainAxis * mainSign;

      return vertical
        ? { x: x + crossAxis, y: y + diffMain }
        : { x: x + diffMain, y: y + crossAxis };
    },
  };
}
