import { onCleanup } from "solid-js";
import { useTooltipContext } from "../Tooltip/Tooltip.context";

export interface UseTooltipTriggerResult {
  ctx: ReturnType<typeof useTooltipContext>;
  /**
   * 把打开/关闭相关的原生事件监听器绑定到真正的 DOM 元素上（通过 ref 调用）。
   * 用 addEventListener 而不是 JSX 的 onXxx prop，是为了不占用这些 prop 名——
   * 调用方可以在 <TooltipTrigger onClick={...} onMouseEnter={...}> 上自由传
   * 自己的原生事件处理，两者是完全独立的两套机制（一个走 Solid 的事件系统，
   * 一个是原生 addEventListener），互不覆盖、都会正常触发。
   */
  attachListeners: (el: Element) => void;
}

/**
 * 封装 TooltipTrigger 需要绑定的所有事件逻辑：
 * - pointerenter/pointerleave（悬停进入/离开）走 openDelay/closeDelay
 * - focus/blur（键盘导航）跳过延迟，立即开关——这是无障碍要求，键盘用户不该被迫等待
 * - Escape 立即关闭，并阻止冒泡（避免被外层其他也监听 Escape 的逻辑重复处理）
 * - mousedown 立即关闭——贴近 shadcn/Radix 的行为：用户开始点击这个元素，
 *   就意味着要跟它交互了，tooltip 这时候还杵在那儿会挡视线，应该立刻让路，
 *   不用等 pointerleave 或者走 closeDelay。
 *
 * mousedown 和 focus 之间有一个容易踩的坑：原生事件顺序是
 * mousedown → focus → mouseup → click——点击一个可聚焦元素（比如 <button>），
 * 浏览器会在 mousedown 之后自动给它 focus。如果不做处理，mousedown 刚关掉
 * tooltip，紧跟着浏览器自动触发的 focus 又会把它立刻重新打开，两条逻辑打架。
 * 用 isPointerDown 标记"这次 focus 是不是紧跟在点击后面被动触发的"，是的话
 * 就跳过重新打开，只有真正的键盘 Tab 导航触发的 focus 才会打开 tooltip。
 */
export function useTooltipTrigger(): UseTooltipTriggerResult {
  const ctx = useTooltipContext("TooltipTrigger");

  const attachListeners = (el: Element) => {
    let isPointerDown = false;

    const onPointerEnter = () => {
      if (!ctx.disabled()) ctx.requestOpen();
    };
    const onPointerLeave = () => {
      ctx.requestClose();
    };
    const onFocus = () => {
      // 紧跟在 mousedown 后面的这次 focus 是浏览器自动带出来的副作用，
      // 不代表用户在用键盘导航，不应该重新打开刚被 mousedown 关掉的 tooltip。
      if (isPointerDown) return;
      if (!ctx.disabled()) ctx.openImmediate();
    };
    const onBlur = () => {
      ctx.closeImmediate();
    };
    const onKeyDown = (e: Event) => {
      const keyboardEvent = e as KeyboardEvent;
      if (keyboardEvent.key === "Escape" && ctx.open()) {
        keyboardEvent.stopPropagation();
        ctx.closeImmediate();
      }
    };
    const onMouseDown = () => {
      isPointerDown = true;
      if (ctx.disabled()) return;
      if (ctx.open()) ctx.closeImmediate();
    };
    // 标记要在 mouseup 时清掉，而且特意监听 document 而不是元素自己的
    // mouseup——如果鼠标按下后拖动到元素外面再松开，元素自己收不到 mouseup，
    // 标记会一直卡在 true，导致后续真正的键盘 focus 也被误伤跳过。
    const onDocumentMouseUp = () => {
      isPointerDown = false;
    };

    el.addEventListener("pointerenter", onPointerEnter);
    el.addEventListener("pointerleave", onPointerLeave);
    el.addEventListener("focus", onFocus);
    el.addEventListener("blur", onBlur);
    el.addEventListener("keydown", onKeyDown);
    el.addEventListener("mousedown", onMouseDown);
    document.addEventListener("mouseup", onDocumentMouseUp);

    onCleanup(() => {
      el.removeEventListener("pointerenter", onPointerEnter);
      el.removeEventListener("pointerleave", onPointerLeave);
      el.removeEventListener("focus", onFocus);
      el.removeEventListener("blur", onBlur);
      el.removeEventListener("keydown", onKeyDown);
      el.removeEventListener("mousedown", onMouseDown);
      document.removeEventListener("mouseup", onDocumentMouseUp);
    });
  };

  return { ctx, attachListeners };
}
