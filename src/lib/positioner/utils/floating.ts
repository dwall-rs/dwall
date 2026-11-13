import { getAlignment, getSide } from "../core/placement";
import {
  arrow,
  containingBlockOffset,
  flip,
  hide,
  offset,
  shift,
} from "../middleware";
import type { Alignment, Middleware, Placement, Side } from "../types";

/** Base UI 的逻辑方向:按 dir 映射到物理方向 */
export type LogicalSide = Side | "inline-start" | "inline-end";

export function toPhysicalSide(side: LogicalSide, dir?: string): Side {
  if (side === "inline-start") return dir === "rtl" ? "right" : "left";
  if (side === "inline-end") return dir === "rtl" ? "left" : "right";
  return side;
}

/** shadcn/Base UI 风格的 side + align 组合成核心库的 Placement */
export function toPlacement(
  side: LogicalSide,
  align: Alignment | "center",
  dir?: string,
): Placement {
  const physical = toPhysicalSide(side, dir);
  return align === "center" ? physical : (`${physical}-${align}` as Placement);
}

/** 缩放动画锚点落在贴近 trigger 的那条边(不是几何中心) */
export function getTransformOrigin(placement: Placement): string {
  const side = getSide(placement);
  const align = getAlignment(placement);
  const cross = align === "start" ? "0%" : align === "end" ? "100%" : "50%";
  switch (side) {
    case "top":
      return `${cross} 100%`;
    case "bottom":
      return `${cross} 0%`;
    case "left":
      return `100% ${cross}`;
    case "right":
      return `0% ${cross}`;
  }
}

export interface FloatingMiddlewareOptions {
  /** 沿主轴与 reference 的间距 */
  sideOffset: number;
  /** 沿交叉轴的偏移 */
  alignOffset?: number;
  /** 距视口边缘的最小间距 */
  collisionPadding?: number;
  /** 传入则加入 arrow middleware(必须排在 flip/shift 之后) */
  arrowElement?: () => Element | undefined;
  arrowPadding?: number;
}

/**
 * Popover / Tooltip / HoverCard / Select 等浮层共用的定位管线:
 * offset → flip → shift → [arrow] → hide → containingBlockOffset。
 * 抽到 positioner 里,避免每个组件各抄一份。
 */
export function createFloatingMiddleware(
  options: FloatingMiddlewareOptions,
): Middleware[] {
  const middleware: Middleware[] = [
    offset({
      mainAxis: options.sideOffset,
      crossAxis: options.alignOffset ?? 0,
    }),
    flip(),
    shift({ padding: options.collisionPadding ?? 8 }),
  ];
  if (options.arrowElement) {
    middleware.push(
      arrow({
        element: options.arrowElement,
        padding: options.arrowPadding ?? 6,
      }),
    );
  }
  middleware.push(hide(), containingBlockOffset());
  return middleware;
}
