import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "@/lib";
import { useSelectContext } from "../Select/Select.context";
import { createSelectMiddleware } from "./SelectContent.utils";
import type { SelectContentProps } from "./SelectContent.types";

export interface UseSelectContentResult {
  ctx: ReturnType<typeof useSelectContext>;
  pos: Positioner;
  /**
   * 面板应该用的最大高度：有选中值时，直接读 alignSelectedItem 自己算好的
   * panelHeight（不经过 size 中间件）；没有选中值（占位符状态、走贴边下拉）
   * 时，用 size 中间件算出来的可用空间。两种场景用的是不同的高度来源，
   * 统一在这里合并成一个对外的值，展示层不需要关心具体走的是哪一个。
   */
  maxHeight: () => number | undefined;
  /** 综合了 open 状态 + hide 中间件的 referenceHidden 判断 */
  isVisible: () => boolean;
  /** 代表"DOM 里要不要还留着这个节点"，展示层应该用它控制 <Show> */
  mounted: () => boolean;
  /** 驱动 data-state 的状态，打开时比 isVisible 晚一帧，给"选中项对齐"这类
   * 需要先读到子元素（选项）才能算准位置的定位策略留出时间。 */
  animationState: () => "open" | "closed";
  contentElement: () => HTMLElement | undefined;
  setContentElement: (el: HTMLElement) => void;
}

export function useSelectContent(
  props: () => SelectContentProps,
): UseSelectContentResult {
  const ctx = useSelectContext("SelectContent");
  const [contentElement, setContentElement] = createSignal<HTMLElement>();
  const [fallbackMaxHeight, setFallbackMaxHeight] = createSignal<number>();

  const hasValue = createMemo(() => ctx.value() !== undefined);

  const pos = createPositioner(ctx.reference, ctx.floating, {
    placement: () => props().placement ?? "bottom-start",
    strategy: "fixed",
    middleware: () =>
      createSelectMiddleware({
        hasValue: hasValue(),
        selectedValue: ctx.value,
        scrollElement: contentElement,
        placement: props().placement ?? "bottom-start",
        collisionPadding: props().collisionPadding ?? 8,
        onAvailableHeightChange: (availableHeight) => {
          setFallbackMaxHeight(Math.max(availableHeight, 120));
        },
      }),
  });

  const maxHeight = createMemo(() => {
    if (hasValue()) {
      const panelHeight = pos.middlewareData().itemAlign?.panelHeight as
        | number
        | undefined;
      return panelHeight !== undefined ? Math.max(panelHeight, 120) : undefined;
    }
    return fallbackMaxHeight();
  });

  const isVisible = createMemo(
    () => ctx.open() && !pos.middlewareData().hide?.referenceHidden,
  );

  // --- 打开时延迟一帧再触发动画 class ---
  // "选中项对齐"策略要读子元素（SelectItem 渲染出来的 <li data-value>）才能
  // 算出正确的位置，如果动画在这次定位算完之前就开始播放，面板会在动画播到
  // 一半时才"跳"到正确位置——等一帧，给这次定位计算留出时间（细节和
  // production-tooltip 里箭头位置的处理是同一个道理）。
  const [animationState, setAnimationState] = createSignal<"open" | "closed">(
    "closed",
  );

  createEffect(() => {
    if (isVisible()) {
      const raf = requestAnimationFrame(() => setAnimationState("open"));
      onCleanup(() => cancelAnimationFrame(raf));
    } else {
      setAnimationState("closed");
    }
  });

  // 没有退场动画（见 SelectContent.tsx 里的说明），mounted 直接跟 isVisible 走，
  // 不需要 Presence 那套"延迟卸载等动画播完"的逻辑。这不只是简化——是有意的：
  // 选中一项的瞬间，ctx.value() 变化会让定位策略从"贴边下拉"切到"选中项对齐"，
  // 两者算出来的坐标完全不同；如果退场动画期间面板还挂在 DOM 里，这次重新定位
  // 会让面板先跳一下再消失，观感很突兀。立即卸载就没有这个问题。
  const mounted = isVisible;

  return {
    ctx,
    pos,
    maxHeight,
    isVisible,
    mounted,
    animationState,
    contentElement,
    setContentElement,
  };
}
