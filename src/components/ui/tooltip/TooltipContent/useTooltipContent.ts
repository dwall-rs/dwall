import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "~/lib";
import { useTooltipContext } from "../Tooltip/Tooltip.context";
import { createTooltipMiddleware, toPlacement } from "./TooltipContent.utils";
import type { TooltipContentProps } from "./TooltipContent.types";

/** 没有触发任何 CSS animation 时的兜底卸载延迟（ms）——比如没装 tailwindcss-animate、
 * 或者自定义样式只用了 transition 而不是 @keyframes，animationend 永远不会触发，
 * 避免元素卡在 DOM 里退不出去。 */
const EXIT_FALLBACK_MS = 300;

export interface UseTooltipContentResult {
  ctx: ReturnType<typeof useTooltipContext>;
  pos: Positioner;
  /** 综合了 open 状态 + hide 中间件的 referenceHidden 判断，代表"逻辑上应不应该打开" */
  isVisible: () => boolean;
  /**
   * 代表"DOM 里要不要还留着这个节点"。isVisible 变 false 之后，mounted 不会立刻
   * 跟着变 false，而是等退场动画播完（或者兜底超时）才变——展示层应该用它来控制
   * <Show>，而不是直接用 isVisible，否则退场动画根本来不及播放就被卸载了。
   */
  mounted: () => boolean;
  /**
   * 真正用来驱动 data-state（进而驱动 CSS 进出场动画 class）的状态，
   * 展示层应该用它，而不是直接用 isVisible。
   *
   * 和 isVisible 的区别只在"打开"这一侧：isVisible 变 true 的瞬间，
   * animationState 不会立刻跟着变 "open"，而是等一帧（requestAnimationFrame）
   * 之后才变——这一帧的间隙用来让箭头（TooltipArrow）完成挂载、把自己注册给
   * arrow middleware、拿到正确的位置偏移。如果不等这一帧，动画会在箭头位置
   * 还没算对的时候就开始播放，播到一半箭头才"跳"到正确位置，看起来就像是
   * 箭头比内容慢一拍——这正是因为内容的淡入/缩放靠的是 CSS 动画（父元素的
   * opacity/transform 会连带影响所有子元素，包括箭头，两者本该同步），而箭头
   * 的具体位置靠的是 JS 算出来的 left/top，这是两套完全独立的机制，只有当
   * "开始播放动画"这个时间点晚于"箭头位置算完"，才能保证两者观感一致。
   * 关闭这一侧不需要这个延迟，isVisible 变 false 时 animationState 立刻跟着变。
   */
  animationState: () => "open" | "closed";
  arrowElement: () => Element | undefined;
  setArrowElement: (el: Element) => void;
  /** 内层内容元素（真正播放进出场动画的那一层），TooltipContent 需要把它 ref 进来 */
  contentElement: () => HTMLElement | undefined;
  setContentElement: (el: HTMLElement) => void;
  /**
   * 加在"滑动层"（定位层和内容层之间的中间那层）上的 style。
   * 命中 TooltipGroup 抢占时会有一个非零初始偏移，下一帧起会用 CSS transition
   * 平滑归零，从而制造出"从旧 tooltip 位置滑到新位置"的效果；平时是空对象，
   * 不产生任何影响。
   */
  slideStyle: () => { transform?: string; transition?: string };
}

export function useTooltipContent(
  props: () => TooltipContentProps,
): UseTooltipContentResult {
  const ctx = useTooltipContext("TooltipContent");
  const [arrowElement, setArrowElement] = createSignal<Element>();
  const [contentElement, setContentElement] = createSignal<HTMLElement>();

  const pos = createPositioner(ctx.reference, ctx.floating, {
    placement: () =>
      toPlacement(
        props().side ?? "top",
        props().align ?? "center",
        props().dir,
      ),
    strategy: "fixed",
    middleware: () =>
      createTooltipMiddleware({
        sideOffset: props().sideOffset ?? 4,
        alignOffset: props().alignOffset ?? 0,
        collisionPadding: props().collisionPadding ?? 8,
        arrowElement,
      }),
  });

  // reference 完全滚出视口（比如所在容器被单独滚动）时，hide middleware 会标记
  // referenceHidden，这里据此隐藏浮层，避免它悬空显示在一个已经看不到的触发器旁边。
  const isVisible = createMemo(
    () => ctx.open() && !pos.middlewareData().hide?.referenceHidden,
  );

  // --- 箭头和内容同步入场：延迟一帧再触发动画 class ---
  // 见 UseTooltipContentResult.animationState 的注释：只有"开始播放动画"这个
  // 时间点晚于"箭头位置算完"，才能保证箭头不会在动画播到一半时才跳到正确位置。
  const [animationState, setAnimationState] = createSignal<"open" | "closed">(
    "closed",
  );

  createEffect(() => {
    if (isVisible()) {
      const raf = requestAnimationFrame(() => setAnimationState("open"));
      onCleanup(() => cancelAnimationFrame(raf));
    } else {
      // 关闭不需要等——退场动画本来就是从"当前已经落定的位置"往外淡出/缩小，
      // 不存在箭头位置还没算好这个问题，没理由延迟。
      setAnimationState("closed");
    }
  });

  // --- TooltipGroup 抢占时的"滑入"过渡（FLIP 技术） ---
  const [slideOffset, setSlideOffset] = createSignal<{
    x: number;
    y: number;
  }>();

  const [mounted, setMounted] = createSignal(isVisible());

  createEffect(() => {
    if (!mounted()) return;
    const from = ctx.pendingSlideFrom();
    if (!from) return;
    // 必须等真正定位完成（知道自己算出来的最终坐标）才能算出"要从哪滑过来"的差值。
    if (!pos.isPositioned()) return;

    const dx = from.x - pos.x();
    const dy = from.y - pos.y();
    // 第一步：把偏移设成非零值——这一步和 floatingStyles() 的最终坐标叠加后，
    // 视觉上让新 tooltip 一出现就恰好在旧 tooltip 原来的位置，没有任何跳动。
    setSlideOffset({ x: dx, y: dy });
    // 消费一次立刻清空，避免后续 autoUpdate（滚动/resize 触发的重新定位）
    // 又把这次的偏移重新套用一遍。
    ctx.clearPendingSlideFrom();

    // 第二步：下一帧再把偏移量归零。因为这次更新带上了 transition（见下面
    // slideStyle 的实现），浏览器会把"从非零偏移回到 0"这个变化平滑动画出来，
    // 效果就是从旧位置滑到新位置。用 requestAnimationFrame 而不是同步归零，
    // 是为了确保浏览器先真正画出第一帧（非零偏移、无 transition），
    // 下一帧的变化才有"起点"可以过渡，不会被浏览器合并成一次跳变。
    requestAnimationFrame(() => setSlideOffset({ x: 0, y: 0 }));
  });

  const slideStyle = createMemo(() => {
    const offset = slideOffset();
    if (!offset) return {};
    const atRest = offset.x === 0 && offset.y === 0;
    return {
      transform: `translate(${offset.x}px, ${offset.y}px)`,
      // 只有"归零"这一步才带 transition：非零的那一帧要求瞬间落位、不能有动画，
      // 否则会先看到它从视口原点飞过来，而不是从旧 tooltip 的位置飞过来。
      transition: atRest ? "transform 150ms ease" : undefined,
    };
  });

  // --- 退场动画支持（Presence）---
  // isVisible 从 true 变 false 的瞬间，不立刻让 <Show> 卸载节点：先把 data-state
  // 切到 closed（触发退场动画的 CSS class），等这个节点自己的 animationend 触发
  // （或者兜底超时）之后，才真正把 mounted 设为 false，交给 <Show> 卸载。
  createEffect((wasVisible: boolean) => {
    const visible = isVisible();
    const el = contentElement();

    if (visible) {
      setMounted(true);
    } else if (wasVisible && el) {
      if (ctx.consumeSuppressExitAnimation()) {
        // 被 TooltipGroup 抢占强制关闭：跳过退场动画，立即卸载，避免和正在
        // 滑入的新 tooltip 同时出现在屏幕上（这是"同时存在两个 tooltip"这个
        // bug 的根因——之前不管什么原因关闭，都统一走了退场动画那条路）。
        setMounted(false);
      } else {
        const handleAnimationEnd = (e: AnimationEvent) => {
          // 忽略从子元素冒泡上来的 animationend，只认内容元素自己播放的那个动画
          if (e.target !== el) return;
          setMounted(false);
        };
        el.addEventListener("animationend", handleAnimationEnd);
        onCleanup(() =>
          el.removeEventListener("animationend", handleAnimationEnd),
        );

        const fallback = window.setTimeout(
          () => setMounted(false),
          EXIT_FALLBACK_MS,
        );
        onCleanup(() => window.clearTimeout(fallback));
      }
    } else if (!visible) {
      setMounted(false);
    }

    return visible;
  }, isVisible());

  return {
    ctx,
    pos,
    isVisible,
    mounted,
    animationState,
    arrowElement,
    setArrowElement,
    contentElement,
    setContentElement,
    slideStyle,
  };
}
