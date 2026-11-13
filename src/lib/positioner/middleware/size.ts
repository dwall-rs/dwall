import type { Boundary, Middleware, MiddlewareState } from "../types";
import { getViewportBoundary } from "../utils/dom";

export interface SizeAvailableSpace {
  availableWidth: number;
  availableHeight: number;
}

export interface SizeOptions {
  padding?: number;
  boundary?: Boundary;
  /** 拿到可用空间后自行处理，比如设置 floating 的 maxHeight */
  apply?: (space: SizeAvailableSpace, state: MiddlewareState) => void;
}

/** 计算 floating 在当前边界内的可用宽高，交给 apply 回调去应用（如设置 max-height 实现滚动） */
export function size(options: SizeOptions = {}): Middleware {
  return {
    name: "size",
    fn(state) {
      const { x, y, strategy } = state;
      const { padding = 0 } = options;
      const boundary =
        options.boundary ?? getViewportBoundary(strategy, padding);

      const availableWidth = boundary.x + boundary.width - x;
      const availableHeight = boundary.y + boundary.height - y;

      options.apply?.({ availableWidth, availableHeight }, state);

      return { data: { availableWidth, availableHeight } };
    },
  };
}
