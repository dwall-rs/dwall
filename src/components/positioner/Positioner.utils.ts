import type { EdgeInsets, Rect } from "./Positioner.types";

/** 将 boundaryPadding 规范化为四边值 */
export function normalizePadding(
  p: number | Partial<EdgeInsets> = 8,
): EdgeInsets {
  if (typeof p === "number") return { top: p, right: p, bottom: p, left: p };
  return { top: 0, right: 0, bottom: 0, left: 0, ...p };
}

/** 两个 Rect 的交集（clipping rect） */
export function intersectRects(a: Rect, b: Rect): Rect {
  const top = Math.max(a.top, b.top);
  const left = Math.max(a.left, b.left);
  const right = Math.min(a.left + a.width, b.left + b.width);
  const bottom = Math.min(a.top + a.height, b.top + b.height);
  return {
    top,
    left,
    width: Math.max(0, right - left),
    height: Math.max(0, bottom - top),
  };
}

/** 判断元素是否是滚动容器 */
export function isScrollable(el: HTMLElement): boolean {
  const { overflow, overflowX, overflowY } = getComputedStyle(el);
  return /auto|scroll|hidden|clip/.test(overflow + overflowX + overflowY);
}

/** 向上查找最近的滚动祖先（不含 body/html，它们用 viewport 覆盖） */
function findScrollParent(el: HTMLElement): HTMLElement | null {
  let cur = el.parentElement;
  while (cur && cur !== document.body && cur !== document.documentElement) {
    if (isScrollable(cur)) return cur;
    cur = cur.parentElement;
  }
  return null;
}

/** 获取有效边界 rect（已减去 padding），坐标为 viewport 坐标系 */
export function getClippingRect(
  trigger: HTMLElement,
  boundary: HTMLElement | "viewport" | undefined,
  padding: EdgeInsets,
): Rect {
  const vp: Rect = {
    top: 0,
    left: 0,
    width: window.innerWidth,
    height: window.innerHeight,
  };

  let clipping: Rect = vp;

  if (boundary === "viewport") {
    clipping = vp;
  } else if (boundary instanceof HTMLElement) {
    const r = boundary.getBoundingClientRect();
    clipping = intersectRects(vp, {
      top: r.top,
      left: r.left,
      width: boundary.clientWidth,
      height: boundary.clientHeight,
    });
  } else {
    const scrollParent = findScrollParent(trigger);
    if (scrollParent) {
      const r = scrollParent.getBoundingClientRect();
      clipping = intersectRects(vp, {
        top: r.top,
        left: r.left,
        width: scrollParent.clientWidth,
        height: scrollParent.clientHeight,
      });
    }
  }

  return {
    top: clipping.top + padding.top,
    left: clipping.left + padding.left,
    width: clipping.width - padding.left - padding.right,
    height: clipping.height - padding.top - padding.bottom,
  };
}

/**
 * 获取元素相对于指定容器祖先的累计 offsetTop。
 *
 * 原生 offsetTop 只是相对于直接 offsetParent 的距离，
 * 若 anchorEl 被多层元素包裹（如 li → ul → 浮层），
 * 直接读 offsetTop 只会得到相对 ul 的偏移，而非相对浮层的偏移，
 * 导致锚点坐标计算错误。此函数沿 offsetParent 链累加，直到命中 container。
 */
export function getAbsoluteOffsetTop(
  el: HTMLElement,
  container: HTMLElement,
): number {
  let offset = 0;
  let curr: HTMLElement | null = el;
  while (curr && curr !== container) {
    offset += curr.offsetTop;
    curr = curr.offsetParent as HTMLElement | null;
  }
  return offset;
}

/**
 * 获取元素相对于指定容器祖先的累计 offsetLeft。
 * 与 getAbsoluteOffsetTop 对称。
 */
export function getAbsoluteOffsetLeft(
  el: HTMLElement,
  container: HTMLElement,
): number {
  let offset = 0;
  let curr: HTMLElement | null = el;
  while (curr && curr !== container) {
    offset += curr.offsetLeft;
    curr = curr.offsetParent as HTMLElement | null;
  }
  return offset;
}

/** 找浮层内第一个可滚动子元素 */
export function findScrollableChild(el: HTMLElement): HTMLElement | null {
  for (const child of el.querySelectorAll<HTMLElement>("*")) {
    if (isScrollable(child) && child.scrollHeight > child.clientHeight) {
      return child;
    }
  }
  return null;
}
