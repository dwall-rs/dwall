import type { PositionerProps, Rect } from "./Positioner.types";
import {
  getAbsoluteOffsetLeft,
  getAbsoluteOffsetTop,
} from "./Positioner.utils";

// ─── Normal position ──────────────────────────────────────────────────────────

export function computeNormalPosition(
  trigger: Rect,
  floating: Rect,
  opts: Required<
    Pick<
      PositionerProps,
      "side" | "sideOffset" | "align" | "alignOffset" | "alignItemWithTrigger"
    >
  >,
): { top: number; left: number; width?: number; height?: number } {
  const { side, sideOffset, align, alignOffset, alignItemWithTrigger } = opts;
  let top = 0,
    left = 0;
  let width: number | undefined, height: number | undefined;

  switch (side) {
    case "bottom":
      top = trigger.top + trigger.height + sideOffset;
      break;
    case "top":
      top = trigger.top - floating.height - sideOffset;
      break;
    case "left":
      left = trigger.left - floating.width - sideOffset;
      break;
    case "right":
      left = trigger.left + trigger.width + sideOffset;
      break;
  }

  const isVertical = side === "top" || side === "bottom";

  if (isVertical) {
    if (alignItemWithTrigger) width = trigger.width;
    const w = alignItemWithTrigger ? trigger.width : floating.width;
    switch (align) {
      case "start":
        left = trigger.left + alignOffset;
        break;
      case "center":
        left = trigger.left + (trigger.width - w) / 2 + alignOffset;
        break;
      case "end":
        left = trigger.left + trigger.width - w - alignOffset;
        break;
    }
  } else {
    if (alignItemWithTrigger) height = trigger.height;
    const h = alignItemWithTrigger ? trigger.height : floating.height;
    switch (align) {
      case "start":
        top = trigger.top + alignOffset;
        break;
      case "center":
        top = trigger.top + (trigger.height - h) / 2 + alignOffset;
        break;
      case "end":
        top = trigger.top + trigger.height - h - alignOffset;
        break;
    }
  }

  return { top, left, width, height };
}

// ─── Anchor position ──────────────────────────────────────────────────────────

/**
 * 锚点对齐模式的坐标计算。
 *
 * 两条轴的计算逻辑刻意分离，原因如下：
 *
 * 【主轴（anchorSide 方向）】
 *   需要让 anchorEl 的某条边与 trigger 的对应边在视觉上重合。
 *   使用 getAbsoluteOffsetTop/Left（相对浮层的累计布局偏移），
 *   这是稳定的相对坐标，不受浮层当前 viewport 位置影响，第一帧即准确。
 *
 *   注意：anchorEl 可能被多层元素包裹，必须沿 offsetParent 链累加，
 *   而不能直接用 anchorEl.offsetTop（只是相对直接父级的偏移）。
 *
 *   注意：offsetTop 不含浮层自身的 border-top，需加上 floatingEl.clientTop 修正。
 *
 * 【交叉轴（anchorAlign 方向）】
 *   需要让「浮层边框」与 trigger 对齐，而不是让 anchorEl 内容边对齐。
 *   直接将浮层的外边缘对齐 trigger，不依赖 anchorEl 的位置。
 */
export function computeAnchorPosition(
  trigger: Rect,
  floatingEl: HTMLElement,
  anchorEl: HTMLElement,
  anchorSide: NonNullable<PositionerProps["anchorSide"]>,
  anchorAlign: NonNullable<PositionerProps["anchorAlign"]>,
): { top: number; left: number } {
  // 用 getAbsoluteOffset* 沿 offsetParent 链累加，支持多层嵌套包裹
  // clientTop/clientLeft：浮层自身的 border 宽度（offsetTop 不含 border，需补偿）
  const itemOffsetTop =
    getAbsoluteOffsetTop(anchorEl, floatingEl) - floatingEl.clientTop;
  const itemOffsetLeft =
    getAbsoluteOffsetLeft(anchorEl, floatingEl) - floatingEl.clientLeft;
  const itemHeight = anchorEl.offsetHeight;
  const itemWidth = anchorEl.offsetWidth;

  const floatingWidth = floatingEl.offsetWidth;
  const floatingHeight = floatingEl.offsetHeight;

  let top = 0,
    left = 0;

  // ── 主轴：anchorEl 的边与 trigger 的边对齐 ──────────────────────────────
  switch (anchorSide) {
    case "top":
      // anchorEl.top === trigger.top  →  floatingTop = trigger.top - itemOffsetTop
      top = trigger.top - itemOffsetTop;
      break;
    case "bottom":
      // anchorEl.bottom === trigger.bottom
      top = trigger.top + trigger.height - itemOffsetTop - itemHeight;
      break;
    case "left":
      left = trigger.left - itemOffsetLeft;
      break;
    case "right":
      left = trigger.left + trigger.width - itemOffsetLeft - itemWidth;
      break;
  }

  const isVerticalSide = anchorSide === "top" || anchorSide === "bottom";

  // ── 交叉轴：浮层外边框直接对齐 trigger ──────────────────────────────────
  if (isVerticalSide) {
    switch (anchorAlign) {
      case "start":
        left = trigger.left;
        break;
      case "center":
        left = trigger.left + (trigger.width - floatingWidth) / 2;
        break;
      case "end":
        left = trigger.left + trigger.width - floatingWidth;
        break;
    }
  } else {
    switch (anchorAlign) {
      case "start":
        top = trigger.top;
        break;
      case "center":
        top = trigger.top + (trigger.height - floatingHeight) / 2;
        break;
      case "end":
        top = trigger.top + trigger.height - floatingHeight;
        break;
    }
  }

  return { top, left };
}
