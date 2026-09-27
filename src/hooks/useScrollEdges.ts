import { createEffect, createSignal, onCleanup, type Accessor } from "solid-js";

export interface ScrollEdges {
  /** Whether content above is still hidden out of view */
  canScrollUp: Accessor<boolean>;
  /** Whether content below is still hidden out of view */
  canScrollDown: Accessor<boolean>;
  /** Manually refresh the edge state once (e.g. after opening or once content changes) */
  refresh: () => void;
}

/**
 * Track whether a scroll container still has hidden content above/below, driving the scroll hint arrows.
 *
 * Listens to scroll(passive) + ResizeObserver(container and first-child size) +
 * MutationObserver(childList/subtree), so visibility updates promptly when the panel height changes
 * or options are added/removed.
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
