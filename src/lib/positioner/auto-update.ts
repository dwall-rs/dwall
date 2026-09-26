import type { ReferenceElement } from "./types";
import { getOverflowAncestors } from "./utils/dom";

export interface AutoUpdateOptions {
  /** 祖先滚动容器滚动时触发更新，默认 true */
  ancestorScroll?: boolean;
  /** 祖先容器 resize（依赖 ResizeObserver）时触发更新，默认 true */
  ancestorResize?: boolean;
  /** reference / floating 自身尺寸变化时触发更新，默认 true（虚拟参照元素没有尺寸，会被自动跳过） */
  elementResize?: boolean;
}

/**
 * 自动监听所有可能影响定位的事件（滚动、窗口/元素尺寸变化），
 * 在变化时调用 update。返回一个清理函数，调用后移除所有监听。
 *
 * reference 如果是虚拟参照元素（比如锚定到鼠标坐标的右键菜单），
 * 没有真实的 DOM 节点可以查找祖先滚动容器或做尺寸观察：
 * - 如果虚拟元素提供了 `contextElement`，会用它代替去查找可滚动祖先；
 * - 否则只监听 window 的 resize（不含 scroll，因为没有关联容器可判断是否相关）。
 */
export function autoUpdate(
  reference: ReferenceElement,
  floating: HTMLElement,
  update: () => void,
  options: AutoUpdateOptions = {},
): () => void {
  const {
    ancestorScroll = true,
    ancestorResize = true,
    elementResize = true,
  } = options;

  const isRealElement = reference instanceof Element;
  const ancestorAnchor: Element | null = isRealElement
    ? reference
    : ((reference as { contextElement?: Element }).contextElement ?? null);

  const ancestors =
    (ancestorScroll || ancestorResize) && ancestorAnchor
      ? getOverflowAncestors(ancestorAnchor)
      : [];

  ancestors.forEach((ancestor) => {
    if (ancestorScroll) {
      ancestor.addEventListener("scroll", update, { passive: true });
    }
  });

  let resizeObserver: ResizeObserver | null = null;
  if (
    (ancestorResize || elementResize) &&
    typeof ResizeObserver !== "undefined"
  ) {
    resizeObserver = new ResizeObserver(() => update());
    // 虚拟参照元素没有真实尺寸可观察，只观察 floating 自身
    if (elementResize) {
      if (isRealElement) resizeObserver.observe(reference);
      resizeObserver.observe(floating);
    }
    if (ancestorResize) {
      ancestors.forEach((ancestor) => {
        if (ancestor instanceof Element) resizeObserver!.observe(ancestor);
      });
    }
  }

  window.addEventListener("resize", update);

  return () => {
    ancestors.forEach((ancestor) => {
      ancestor.removeEventListener("scroll", update);
    });
    resizeObserver?.disconnect();
    window.removeEventListener("resize", update);
  };
}
