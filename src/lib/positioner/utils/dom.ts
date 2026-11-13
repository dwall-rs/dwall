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
  // 移动端软键盘 / 地址栏变化会改变 visualViewport，但不会改变
  // documentElement.clientHeight。用 visualViewport 作为边界，浮层才不会
  // 被定位到键盘后面；没有 visualViewport 的环境回退到布局视口。
  const viewport = window.visualViewport;
  const width = viewport?.width ?? document.documentElement.clientWidth;
  const height = viewport?.height ?? document.documentElement.clientHeight;
  const offsetX = viewport?.offsetLeft ?? 0;
  const offsetY = viewport?.offsetTop ?? 0;
  const scrollX =
    strategy === "fixed" ? 0 : (window.scrollX ?? window.pageXOffset);
  const scrollY =
    strategy === "fixed" ? 0 : (window.scrollY ?? window.pageYOffset);
  return {
    x: scrollX + offsetX + padding,
    y: scrollY + offsetY + padding,
    width: width - padding * 2,
    height: height - padding * 2,
  };
}

/**
 * 元素自身是否建立了滚动容器。
 *
 * 导出原因:`hooks/useScrollLock` 需要用它判断页面滚动条长在 html 还是 body 上,
 * 之前那里也有一份逐字重复的实现。两处口径必须一致(同一个正则、同样只看
 * overflow 三属性),否则"锁哪个元素"和"哪些祖先要监听 scroll"会不一致。
 */
export function isOverflowElement(el: Element): boolean {
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
