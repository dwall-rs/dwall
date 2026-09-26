import type { Boundary, Middleware } from "../types";
import { getViewportBoundary } from "../utils/dom";

export interface ShiftOptions {
  /** 距边界的最小间距，默认 0 */
  padding?: number;
  /** 自定义边界矩形，默认使用视口 */
  boundary?: Boundary;
  /** 是否限制主轴方向（如 top/bottom placement 的 y 方向），默认 true */
  mainAxis?: boolean;
  /** 是否限制交叉轴方向（如 top/bottom placement 的 x 方向），默认 true */
  crossAxis?: boolean;
}

/** 平移 floating 元素，使其尽量停留在边界（默认视口）内，不做 placement 翻转 */
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
