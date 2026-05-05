import {
  createEffect,
  createSignal,
  onCleanup,
  onMount,
  untrack,
} from "solid-js";
import { computePosition } from "./TooltipContent.utils";
import { createStore } from "solid-js/store";
import { useTooltipContext } from "../Tooltip/Tooltip.context";
import type { TooltipContentProps } from "./TooltipContent.types";

export const useTooltipContent = (
  ref: () => HTMLDivElement | undefined,
  local: () => Required<
    Pick<TooltipContentProps, "side" | "align" | "alignOffset" | "sideOffset">
  >,
) => {
  const ctx = useTooltipContext();

  const [position, setPosition] = createStore({
    top: 0,
    left: 0,
    side: untrack(() => local().side),
    align: untrack(() => local().align),
  });
  const [shouldRender, setShouldRender] = createSignal(untrack(ctx.open));

  const updatePosition = (content: HTMLElement | undefined) => {
    if (!ctx.triggerRef || !content) return;

    const { top, left, side, align } = computePosition(
      ctx.triggerRef,
      content,
      local().side,
      local().align,
      local().alignOffset,
      local().sideOffset,
    );
    setPosition({ top, left, side, align });
  };

  const updatePositionWithContentRef = () => updatePosition(ref());

  onMount(() => {
    window.addEventListener("scroll", updatePositionWithContentRef, {
      passive: true,
    });
    window.addEventListener("resize", updatePositionWithContentRef, {
      passive: true,
    });
  });

  onCleanup(() => {
    window.removeEventListener("scroll", updatePositionWithContentRef);
    window.removeEventListener("resize", updatePositionWithContentRef);
  });

  createEffect(() => {
    const isOpen = ctx.open();

    if (isOpen) {
      setShouldRender(true);
      // 等待 DOM 完成布局后再计算，双 rAF 确保内容节点已完成首次渲染
      requestAnimationFrame(() => {
        requestAnimationFrame(() => updatePosition(ref()));
      });
      return;
    }

    const el = ref();
    if (!el) {
      setShouldRender(false);
      return;
    }

    const onFinish = () => setShouldRender(false);
    el.addEventListener("animationend", onFinish, { once: true });
    el.addEventListener("transitionend", onFinish, { once: true });
  });

  return { shouldRender, position };
};
