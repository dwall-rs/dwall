import type { FloatPosition, Rect } from "./Positioner.types";

// ─── Flip ─────────────────────────────────────────────────────────────────────

/**
 * Flip 策略：当主轴空间不足时，将浮层翻转到对侧。
 *
 * 返回修正后的 top/left 和 resolvedSide。
 * 不修改交叉轴坐标（由 shift 处理）。
 */
function applyFlip(
  initial: { top: number; left: number },
  floating: { width: number; height: number },
  trigger: Rect,
  clip: Rect,
  side: "top" | "bottom" | "left" | "right",
  sideOffset: number,
): { top: number; left: number; resolvedSide: typeof side } {
  let { top, left } = initial;
  let resolvedSide = side;
  const isVertical = side === "top" || side === "bottom";

  if (isVertical) {
    const spaceBelow =
      clip.top + clip.height - (trigger.top + trigger.height) - sideOffset;
    const spaceAbove = trigger.top - clip.top - sideOffset;

    const fitsBelow = floating.height <= spaceBelow;
    const fitsAbove = floating.height <= spaceAbove;

    if (
      side === "bottom" &&
      !fitsBelow &&
      (fitsAbove || spaceAbove > spaceBelow)
    ) {
      resolvedSide = "top";
      top = trigger.top - floating.height - sideOffset;
    } else if (
      side === "top" &&
      !fitsAbove &&
      (fitsBelow || spaceBelow > spaceAbove)
    ) {
      resolvedSide = "bottom";
      top = trigger.top + trigger.height + sideOffset;
    }
  } else {
    const spaceRight =
      clip.left + clip.width - (trigger.left + trigger.width) - sideOffset;
    const spaceLeft = trigger.left - clip.left - sideOffset;

    const fitsRight = floating.width <= spaceRight;
    const fitsLeft = floating.width <= spaceLeft;

    if (
      side === "right" &&
      !fitsRight &&
      (fitsLeft || spaceLeft > spaceRight)
    ) {
      resolvedSide = "left";
      left = trigger.left - floating.width - sideOffset;
    } else if (
      side === "left" &&
      !fitsLeft &&
      (fitsRight || spaceRight > spaceLeft)
    ) {
      resolvedSide = "right";
      left = trigger.left + trigger.width + sideOffset;
    }
  }

  return { top, left, resolvedSide };
}

// ─── Shift ────────────────────────────────────────────────────────────────────

/**
 * Shift 策略：沿交叉轴平移浮层，使其回到边界内。
 *
 * flip 之后，浮层在交叉轴方向可能仍然超出边界。
 * 在不超过对侧边界的前提下，尽量将浮层推回可视区。
 */
function applyShift(
  pos: { top: number; left: number },
  floating: { width: number; height: number },
  clip: Rect,
  side: "top" | "bottom" | "left" | "right",
): { top: number; left: number } {
  let { top, left } = pos;
  const isVertical = side === "top" || side === "bottom";

  if (isVertical) {
    const minLeft = clip.left;
    const maxLeft = clip.left + clip.width - floating.width;
    left = Math.min(Math.max(left, minLeft), maxLeft);
  } else {
    const minTop = clip.top;
    const maxTop = clip.top + clip.height - floating.height;
    top = Math.min(Math.max(top, minTop), maxTop);
  }

  return { top, left };
}

// ─── Size ─────────────────────────────────────────────────────────────────────

/**
 * Size 策略：限制浮层的 max-height / max-width 使其不溢出边界。
 *
 * 在经过 flip + shift 后，如果浮层仍超出边界（边界本身就太窄），
 * 则写入 maxHeight 或 maxWidth。
 *
 * 关键修正：
 *   - side="top" 时，浮层底边锚定于 trigger.top - sideOffset（不受 shift 影响）。
 *     可用高度 = trigger.top - sideOffset - clip.top，而非 top + height - clip.top。
 *     后者在 shift 移动 top 后会引入错误的额外高度。
 *   - side="left" 同理，右边锚定于 trigger.left - sideOffset。
 *     可用宽度 = trigger.left - sideOffset - clip.left。
 */
function applySize(
  pos: { top: number; left: number },
  floating: { width: number; height: number },
  trigger: Rect,
  clip: Rect,
  resolvedSide: "top" | "bottom" | "left" | "right",
  sideOffset: number,
): { maxWidth?: number; maxHeight?: number } {
  const isVertical = resolvedSide === "top" || resolvedSide === "bottom";
  const clipBottom = clip.top + clip.height;
  const clipRight = clip.left + clip.width;

  let maxWidth: number | undefined;
  let maxHeight: number | undefined;

  if (isVertical) {
    if (resolvedSide === "bottom") {
      // 浮层顶边锚定在 trigger.bottom + sideOffset
      // 可用高度 = clipBottom - pos.top（shift 后 top 已是浮层实际顶边）
      maxHeight = Math.max(0, clipBottom - pos.top);
    } else {
      // resolvedSide === "top"
      // 浮层底边锚定在 trigger.top - sideOffset（不随 shift 移动）
      // 正确参照点：trigger.top - sideOffset，而不是 pos.top + height
      maxHeight = Math.max(0, trigger.top - sideOffset - clip.top);
    }
    // 交叉轴（水平）：若浮层右边超出，裁剪宽度
    if (pos.left + floating.width > clipRight) {
      maxWidth = Math.max(0, clipRight - pos.left);
    }
  } else {
    if (resolvedSide === "right") {
      // 浮层左边锚定在 trigger.right + sideOffset
      maxWidth = Math.max(0, clipRight - pos.left);
    } else {
      // resolvedSide === "left"
      // 浮层右边锚定在 trigger.left - sideOffset
      maxWidth = Math.max(0, trigger.left - sideOffset - clip.left);
    }
    // 交叉轴（垂直）：若浮层底边超出，裁剪高度
    if (pos.top + floating.height > clipBottom) {
      maxHeight = Math.max(0, clipBottom - pos.top);
    }
  }

  return { maxWidth, maxHeight };
}

// ─── Available space ──────────────────────────────────────────────────────────

/**
 * 计算 --available-height / --available-width CSS 变量的值。
 *
 * 语义：「resolvedSide 方向上，从浮层放置点到边界的原始可用空间」。
 * 不受 shift 后坐标影响，反映该方向上理论最大可用空间，
 * 供浮层内滚动区域用 max-height: var(--available-height) 自行控制。
 */
function computeAvailableSpace(
  pos: { top: number; left: number },
  trigger: Rect,
  clip: Rect,
  resolvedSide: "top" | "bottom" | "left" | "right",
  sideOffset: number,
): { availableWidth: number; availableHeight: number } {
  const isVertical = resolvedSide === "top" || resolvedSide === "bottom";
  const clipBottom = clip.top + clip.height;
  const clipRight = clip.left + clip.width;

  let availableHeight: number;
  let availableWidth: number;

  if (isVertical) {
    availableHeight =
      resolvedSide === "bottom"
        ? Math.max(0, clipBottom - (trigger.top + trigger.height) - sideOffset)
        : Math.max(0, trigger.top - clip.top - sideOffset);
    // 交叉轴可用宽度：shift 后实际 left 到右边界
    availableWidth = Math.max(0, clipRight - pos.left);
  } else {
    availableWidth =
      resolvedSide === "right"
        ? Math.max(0, clipRight - (trigger.left + trigger.width) - sideOffset)
        : Math.max(0, trigger.left - clip.left - sideOffset);
    availableHeight = Math.max(0, clipBottom - pos.top);
  }

  return { availableWidth, availableHeight };
}

// ─── Public API ───────────────────────────────────────────────────────────────

/**
 * 对计算好的初始位置依次应用 flip → shift → size 边界约束。
 *
 * @param initial    初始 top/left（由 computeNormalPosition 或 computeAnchorPosition 给出）
 * @param floating   浮层当前尺寸（宽高，来自 getBoundingClientRect）
 * @param trigger    trigger 的 viewport rect
 * @param clip       有效边界 rect（已含 padding）
 * @param side       props 中声明的 side
 * @param sideOffset 主轴间距
 * @param strategy   启用的策略列表
 */
export function applyOverflowCorrection(
  initial: { top: number; left: number; width?: number; height?: number },
  floating: Rect,
  trigger: Rect,
  clip: Rect,
  side: "top" | "bottom" | "left" | "right",
  sideOffset: number,
  strategy: Array<"flip" | "shift" | "size">,
): FloatPosition {
  const floatingSize = {
    width: initial.width ?? floating.width,
    height: initial.height ?? floating.height,
  };

  // ── 1. Flip ────────────────────────────────────────────────────────────────
  let pos: { top: number; left: number };
  let resolvedSide: typeof side;

  if (strategy.includes("flip")) {
    const flipped = applyFlip(
      initial,
      floatingSize,
      trigger,
      clip,
      side,
      sideOffset,
    );
    pos = { top: flipped.top, left: flipped.left };
    resolvedSide = flipped.resolvedSide;
  } else {
    pos = { top: initial.top, left: initial.left };
    resolvedSide = side;
  }

  // ── 2. Shift ───────────────────────────────────────────────────────────────
  if (strategy.includes("shift")) {
    pos = applyShift(pos, floatingSize, clip, resolvedSide);
  }

  // ── 3. Size ────────────────────────────────────────────────────────────────
  let maxWidth: number | undefined;
  let maxHeight: number | undefined;

  if (strategy.includes("size")) {
    ({ maxWidth, maxHeight } = applySize(
      pos,
      floatingSize,
      trigger,
      clip,
      resolvedSide,
      sideOffset,
    ));
  }

  // ── 4. Available space ────────────────────────────────────────────────────
  const { availableWidth, availableHeight } = computeAvailableSpace(
    pos,
    trigger,
    clip,
    resolvedSide,
    sideOffset,
  );

  return {
    top: pos.top,
    left: pos.left,
    ...(initial.width != null ? { width: initial.width } : {}),
    ...(initial.height != null ? { height: initial.height } : {}),
    ...(maxWidth != null ? { maxWidth } : {}),
    ...(maxHeight != null ? { maxHeight } : {}),
    availableWidth,
    availableHeight,
    resolvedSide,
  };
}
