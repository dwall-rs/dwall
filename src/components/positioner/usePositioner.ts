// usePositioner.ts
import { createEffect, createSignal, onCleanup, type JSX } from "solid-js";
import type {
  PositionerProps,
  RequiredPositionerProps,
} from "./Positioner.types";
import {
  getAbsoluteOffsetTop,
  getClippingRect,
  normalizePadding,
} from "./Positioner.utils";
import {
  computeAnchorPosition,
  computeNormalPosition,
} from "./Positioner.position";
import { applyOverflowCorrection } from "./Positioner.overflow";
import { MEASURING_STYLE } from "./Positioner.consts";

export const usePositioner = ({
  props,
}: {
  props: RequiredPositionerProps &
    Pick<
      PositionerProps,
      "triggerRef" | "style" | "open" | "anchorRef" | "boundary"
    >;
}) => {
  const [floatingRef, setFloatingRef] = createSignal<HTMLDivElement>();
  const [floatingStyle, setFloatingStyle] =
    createSignal<JSX.CSSProperties>(MEASURING_STYLE);

  // ── 核心定位计算 ─────────────────────────────────────────────────────────────

  function measure() {
    const el = floatingRef();
    if (!el || !props.triggerRef) return;

    const triggerRect = props.triggerRef.getBoundingClientRect();
    const floatingRect = el.getBoundingClientRect();
    const padding = normalizePadding(props.boundaryPadding);
    const clip = getClippingRect(props.triggerRef, props.boundary, padding);

    // ── Step 1：计算理想位置 ──────────────────────────────────────────────────

    let initial: { top: number; left: number; width?: number; height?: number };

    if (props.anchorRef) {
      const { top, left } = computeAnchorPosition(
        triggerRect,
        el,
        props.anchorRef,
        props.anchorSide,
        props.anchorAlign,
      );
      initial = { top, left };
    } else {
      initial = computeNormalPosition(
        triggerRect,
        {
          top: floatingRect.top,
          left: floatingRect.left,
          width: floatingRect.width,
          height: floatingRect.height,
        },
        {
          side: props.side,
          sideOffset: props.sideOffset,
          align: props.align,
          alignOffset: props.alignOffset,
          alignItemWithTrigger: props.alignItemWithTrigger,
        },
      );
    }

    // ── Step 2：边界约束 ──────────────────────────────────────────────────────

    const corrected = applyOverflowCorrection(
      initial,
      {
        top: floatingRect.top,
        left: floatingRect.left,
        width: floatingRect.width,
        height: floatingRect.height,
      },
      triggerRect,
      clip,
      props.side,
      props.sideOffset,
      props.overflowStrategy,
    );

    // ── Step 3：anchorRef 溢出检测与定位策略 ─────────────────────────────────

    const isAnchorMode = !!props.anchorRef;
    const isVerticalAnchor =
      props.anchorSide === "top" || props.anchorSide === "bottom";

    const marginStyle: JSX.CSSProperties = {};
    if (isAnchorMode && props.sideOffset) {
      if (isVerticalAnchor) {
        marginStyle["margin-top"] = `${props.sideOffset}px`;
        marginStyle["margin-bottom"] = `${props.sideOffset}px`;
      } else {
        marginStyle["margin-left"] = `${props.sideOffset}px`;
        marginStyle["margin-right"] = `${props.sideOffset}px`;
      }
    }

    let positionProps: JSX.CSSProperties;
    // 浮层定位完成后需要写入的 scrollTop（null 表示无需滚动）
    let pendingScroll: { el: HTMLElement; top: number } | null = null;

    if (isAnchorMode && isVerticalAnchor && props.anchorRef) {
      const itemTop = getAbsoluteOffsetTop(props.anchorRef, el) - el.clientTop;
      const idealTop = triggerRect.top - itemTop;
      const idealBottom = idealTop + floatingRect.height;
      const clipBottom = clip.top + clip.height;

      const overflowsTop = idealTop < clip.top;
      const overflowsBottom = idealBottom > clipBottom;

      if (overflowsTop || overflowsBottom) {
        // 浮层高度填满整个裁剪区
        const height = clipBottom - clip.top;
        // 顶部溢出时贴顶，底部溢出时贴底
        const top = overflowsTop ? clip.top : clipBottom - height;

        positionProps = {
          top: `${top}px`,
          left: `${corrected.left}px`,
          height: `${Math.max(0, height)}px`,
          ...(corrected.maxWidth != null
            ? { "max-width": `${corrected.maxWidth}px` }
            : {}),
        };

        // scrollTop 补偿：浮层贴边后，通过滚动让 anchorEl 与 trigger 对齐。
        //
        // 推导（anchorSide="top" 为例，使 anchorEl 顶边 === triggerRect.top）：
        //   anchorEl.viewportTop = top + itemTop - scrollTop
        //   期望 = triggerRect.top
        //   → scrollTop = top + itemTop - triggerRect.top
        //
        // 此时 itemTop 是在 MEASURING_STYLE（无约束）下测量的。
        // 与两阶段方案不同，这里浮层高度 = clipHeight = floatingRect.height 的最大值，
        // 内部布局不会因高度压缩而重排（浮层内容的自然高度 ≥ clipHeight），
        // 所以 itemTop 在约束前后保持不变，无需两阶段测量。
        const scrollEl =
          el.querySelector<HTMLElement>("[data-scroll]") ??
          findScrollableDescendant(el) ??
          el;

        pendingScroll = {
          el: scrollEl,
          top: top + itemTop - triggerRect.top,
        };
      } else {
        positionProps = {
          top: `${idealTop}px`,
          left: `${corrected.left}px`,
        };
      }
    } else {
      positionProps = {
        top: `${corrected.top}px`,
        left: `${corrected.left}px`,
        ...(corrected.width != null ? { width: `${corrected.width}px` } : {}),
        ...(corrected.height != null
          ? { height: `${corrected.height}px` }
          : {}),
        ...(corrected.maxWidth != null
          ? { "max-width": `${corrected.maxWidth}px` }
          : {}),
        ...(corrected.maxHeight != null
          ? { "max-height": `${corrected.maxHeight}px` }
          : {}),
      };
    }

    setFloatingStyle({
      position: "fixed",
      ...positionProps,
      ...marginStyle,
      "--anchor-width": `${triggerRect.width}px`,
      "--anchor-height": `${triggerRect.height}px`,
      "--available-width": `${corrected.availableWidth}px`,
      "--available-height": `${corrected.availableHeight}px`,
      ...props.style,
    } as JSX.CSSProperties);

    el.dataset.side = corrected.resolvedSide;
    el.dataset.align = props.align;

    // ── Step 4：写入 scrollTop ────────────────────────────────────────────────
    //
    // 必须在 setFloatingStyle 之后的独立 rAF 执行：
    // SolidJS 的信号更新是批量异步的，setFloatingStyle 调用后 DOM 还未更新，
    // scrollHeight 仍是旧值。等下一帧浏览器完成 layout 后再写 scrollTop 才有效。
    if (pendingScroll !== null) {
      const { el: scrollEl, top: targetTop } = pendingScroll;
      requestAnimationFrame(() => {
        scrollEl.scrollTop = Math.max(0, targetTop);
      });
    }
  }

  // ── 定位测量 + 响应式更新 ────────────────────────────────────────────────────

  createEffect(() => {
    void props.side;
    void props.sideOffset;
    void props.align;
    void props.alignOffset;
    void props.alignItemWithTrigger;
    void props.anchorSide;
    void props.anchorAlign;
    void props.boundary;
    void props.boundaryPadding;
    void props.overflowStrategy;
    void props.anchorRef;
    void props.triggerRef;

    const floatingEl = floatingRef();
    if (!floatingEl || !props.triggerRef) return;

    setFloatingStyle(MEASURING_STYLE);
    requestAnimationFrame(measure);
  });

  // ── 事件监听（ResizeObserver + scroll/resize）────────────────────────────────
  //
  // scroll 事件必须过滤浮层内部的滚动：
  // 若不过滤，用户在浮层内手动滚动 → 触发 measure() → scrollTop 被重置 → 死锁。
  // 只响应浮层之外的滚动（祖先容器滚动、window 滚动），这些确实会改变 trigger 的位置。

  createEffect(() => {
    const el = floatingRef();
    if (!el) return;

    const ro = new ResizeObserver(measure);
    if (props.triggerRef) ro.observe(props.triggerRef);
    if (props.anchorRef) ro.observe(props.anchorRef);
    ro.observe(el);
    if (props.boundary instanceof HTMLElement) ro.observe(props.boundary);

    function handleScroll(e: Event) {
      const floatingEl = floatingRef();
      if (!floatingEl) return;
      if (e.target instanceof Node && floatingEl.contains(e.target)) return;
      measure();
    }

    window.addEventListener("scroll", handleScroll, true);
    window.addEventListener("resize", measure);

    onCleanup(() => {
      ro.disconnect();
      window.removeEventListener("scroll", handleScroll, true);
      window.removeEventListener("resize", measure);
    });
  });

  return { ref: setFloatingRef, style: floatingStyle };
};

// ── 工具函数 ──────────────────────────────────────────────────────────────────

/**
 * 找浮层内第一个"实际可滚动"的后代（overflow:auto/scroll 且内容超出）。
 */
function findScrollableDescendant(container: HTMLElement): HTMLElement | null {
  for (const child of container.querySelectorAll<HTMLElement>("*")) {
    const { overflow, overflowX, overflowY } = getComputedStyle(child);
    if (
      /auto|scroll/.test(overflow + overflowX + overflowY) &&
      child.scrollHeight > child.clientHeight
    ) {
      return child;
    }
  }
  return null;
}
