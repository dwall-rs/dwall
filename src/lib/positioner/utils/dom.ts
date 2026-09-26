import type { ReferenceElement, Rect, Strategy } from "../types";

/** 把 DOMRect 转成纯数据对象（视口坐标系）；同时兼容虚拟参照元素 */
export function getViewportRect(el: ReferenceElement): Rect {
  const rect = el.getBoundingClientRect();
  return { x: rect.x, y: rect.y, width: rect.width, height: rect.height };
}

/**
 * 根据 strategy 得到参与计算的矩形：
 * - fixed：直接用视口坐标（配合 position:fixed 使用最简单可靠）
 * - absolute：加上页面滚动偏移，换算成“文档坐标”
 *   （假设 floating 元素最终被挂载在 <body> 下，即最近定位祖先是 body/html，
 *    这是 Portal 到 body 的常见用法；如果你的 floating 元素有自定义的
 *    非 static 定位祖先，请优先使用 strategy: 'fixed'）
 */
export function getRectRelativeTo(
  el: ReferenceElement,
  strategy: Strategy,
): Rect {
  const rect = getViewportRect(el);
  if (strategy === "fixed") return rect;
  const scrollX = window.scrollX ?? window.pageXOffset;
  const scrollY = window.scrollY ?? window.pageYOffset;
  return {
    x: rect.x + scrollX,
    y: rect.y + scrollY,
    width: rect.width,
    height: rect.height,
  };
}

/** 当前视口作为默认边界（用于 shift / flip 的溢出检测），strategy 决定坐标系 */
export function getViewportBoundary(strategy: Strategy, padding = 0): Rect {
  const width = document.documentElement.clientWidth;
  const height = document.documentElement.clientHeight;
  const scrollX =
    strategy === "fixed" ? 0 : (window.scrollX ?? window.pageXOffset);
  const scrollY =
    strategy === "fixed" ? 0 : (window.scrollY ?? window.pageYOffset);
  return {
    x: scrollX + padding,
    y: scrollY + padding,
    width: width - padding * 2,
    height: height - padding * 2,
  };
}

function isOverflowElement(el: Element): boolean {
  const { overflow, overflowX, overflowY } = getComputedStyle(el);
  return /auto|scroll|overlay|hidden/.test(overflow + overflowX + overflowY);
}

/** 找到所有可能影响布局的可滚动祖先 + window，用于 autoUpdate 监听 scroll/resize */
export function getOverflowAncestors(node: Element): Array<Element | Window> {
  const result: Array<Element | Window> = [];
  let el: Element | null = node.parentElement;

  while (el) {
    if (isOverflowElement(el)) {
      result.push(el);
    }
    el = el.parentElement;
  }

  result.push(window);
  return result;
}
