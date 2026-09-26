import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "~/lib";
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
  /**
   * 兼容字段：SelectContent 现在关闭时不再卸载 Portal（为了保住 ctx.items 的
   * 注册信息），因此 mounted 已不再被展示层用来控制 <Show>。该值目前等价于
   * isVisible，保留只是为了不破坏已导出的 useSelectContent 返回类型。
   */
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

  const hasValue = createMemo(() => ctx.value() != null);

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

  // SelectContent 关闭时也不会卸载（为了保住 ctx.items 的注册信息），因此
  // createPositioner 不会像 Tooltip/Popover 那样在每次打开时重新创建并自动
  // 算一次位置。这里在每次打开时显式重新计算一次，修复“触发器第一次打开后
  // 因外部布局变化移动了位置，第二次打开时面板还停留在旧坐标”的问题。
  createEffect(() => {
    if (ctx.open()) {
      pos.update();
    }
  });

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

  // SelectContent 关闭时不再卸载子节点（否则 ctx.items 会被清空，SelectValue
  // 读不到 label），因此 mounted 已经没有"延迟卸载/立即卸载"的语义，直接跟
  // isVisible 保持一致即可。保留该字段只是为了兼容已导出的 useSelectContent
  // 返回类型；新的展示层逻辑一律使用 isVisible。
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
