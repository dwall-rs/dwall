import type { Middleware } from "../types";
import { getSide, isVerticalSide } from "../core/placement";

export interface OffsetValue {
  /** 沿主轴方向的偏移（远离/靠近 reference），默认 0 */
  mainAxis?: number;
  /** 沿交叉轴方向的偏移（左右或上下平移），默认 0 */
  crossAxis?: number;
}

export type OffsetOptions = number | OffsetValue;

/** 让 floating 元素与 reference 保持一定间距，或沿对齐轴做微调 */
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

      // top/left 是"负方向"，bottom/right 是"正方向"
      const mainSign = side === "bottom" || side === "right" ? 1 : -1;

      const diffMain = mainAxis * mainSign;

      return vertical
        ? { x: x + crossAxis, y: y + diffMain }
        : { x: x + diffMain, y: y + crossAxis };
    },
  };
}
