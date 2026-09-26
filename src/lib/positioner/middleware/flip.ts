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
  /** 翻转候选 placement 列表，按顺序尝试，默认只翻转到正对面 */
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

/** 取某个 placement 主轴方向对应的溢出量（正数代表溢出了多少像素） */
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

/** 当主轴方向空间不够时，翻转到相对的 placement（如 bottom -> top） */
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

      // 已经翻转过一次就不再继续翻，避免来回震荡
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

      // 当前方向够放，不需要翻
      if (currentMainOverflow <= 0) return {};

      const candidates = options.fallbackPlacements ?? [
        getOppositePlacement(initialPlacement),
      ];

      let best: { placement: Placement; overflow: number } | null = null;

      for (const candidate of candidates) {
        if (candidate === placement) continue;

        // 关键修复点：真正代入候选 placement 算出它的坐标，再算它自己的真实溢出量，
        // 而不是用当前（本就溢出的）placement 的 overflow 去做错误的镜像估算。
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
          break; // 找到完全不溢出的候选，直接采用
        }
        if (!best || candidateMainOverflow < best.overflow) {
          best = { placement: candidate, overflow: candidateMainOverflow };
        }
      }

      // 没有候选，或翻过去也不比现在好，就不翻
      if (!best || best.overflow >= currentMainOverflow) return {};

      return {
        reset: { placement: best.placement },
        data: { flipped: true },
      };
    },
  };
}
