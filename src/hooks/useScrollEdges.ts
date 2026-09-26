import { createEffect, createSignal, onCleanup, type Accessor } from "solid-js";

export interface ScrollEdges {
  /** 上方是否还有未显示的内容 */
  canScrollUp: Accessor<boolean>;
  /** 下方是否还有未显示的内容 */
  canScrollDown: Accessor<boolean>;
  /** 手动刷新一次边缘状态(如打开、内容变化后) */
  refresh: () => void;
}

/**
 * 跟踪某个滚动容器上方/下方是否还有未显示的内容，用于驱动滚动提示箭头。
 *
 * 监听 scroll(passive) + ResizeObserver(容器与首个子元素尺寸) +
 * MutationObserver(childList/subtree)，因此面板高度变化、选项增删后显隐都能及时更新。
 */
export function useScrollEdges(
  target: Accessor<HTMLElement | undefined>,
): ScrollEdges {
  const [canScrollUp, setCanScrollUp] = createSignal(false);
  const [canScrollDown, setCanScrollDown] = createSignal(false);

  const refresh = () => {
    const el = target();
    if (!el) {
      setCanScrollUp(false);
      setCanScrollDown(false);
      return;
    }
    setCanScrollUp(el.scrollTop > 1);
    setCanScrollDown(el.scrollTop + el.clientHeight < el.scrollHeight - 1);
  };

  createEffect(() => {
    const el = target();
    if (!el) return;

    refresh();

    const resizeObserver = new ResizeObserver(refresh);
    resizeObserver.observe(el);
    if (el.firstElementChild) resizeObserver.observe(el.firstElementChild);

    const mutationObserver = new MutationObserver(refresh);
    mutationObserver.observe(el, { childList: true, subtree: true });

    el.addEventListener("scroll", refresh, { passive: true });
    onCleanup(() => {
      resizeObserver.disconnect();
      mutationObserver.disconnect();
      el.removeEventListener("scroll", refresh);
    });
  });

  return { canScrollUp, canScrollDown, refresh };
}
