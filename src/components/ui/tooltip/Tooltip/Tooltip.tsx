import {
  createSignal,
  createMemo,
  createUniqueId,
  createEffect,
  onCleanup,
  type JSX,
} from "solid-js";
import { TooltipContext } from "./Tooltip.context";
import { useTooltipProviderContext } from "../TooltipProvider/TooltipProvider.context";
import { useTooltipGroupContext } from "../TooltipGroup/TooltipGroup.context";
import type { TooltipContextValue, TooltipProps } from "./Tooltip.types";
import type { Rect } from "~/lib";

const DEFAULT_OPEN_DELAY = 0;
const DEFAULT_CLOSE_DELAY = 150;

/**
 * Tooltip 根组件：不渲染任何 DOM，只负责状态管理 + 提供 context。
 * 真正的展示交给 <TooltipTrigger>/<TooltipContent>/<TooltipArrow>。
 *
 * 如果外面包了 <TooltipGroup>，组内相邻 trigger 之间切换 hover 会跳过
 * openDelay，并触发"从旧位置滑到新位置"的过渡（细节见 TooltipGroup.context.ts）。
 *
 * @example
 * ```tsx
 * <Tooltip openDelay={300}>
 *   <TooltipTrigger>悬停我</TooltipTrigger>
 *   <TooltipContent>
 *     提示内容
 *     <TooltipArrow />
 *   </TooltipContent>
 * </Tooltip>
 * ```
 */
export function Tooltip(props: TooltipProps): JSX.Element {
  const group = useTooltipGroupContext(); // undefined 是正常情况，代表没用 TooltipGroup
  const provider = useTooltipProviderContext();

  const [internalOpen, setInternalOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const open = createMemo(() =>
    props.open !== undefined ? props.open : internalOpen(),
  );
  const disabled = createMemo(() => !!props.disabled);

  const [reference, setReference] = createSignal<Element>();
  const [floating, setFloating] = createSignal<HTMLElement>();
  const [pendingSlideFrom, setPendingSlideFrom] = createSignal<Rect>();
  const [suppressExitAnimation, setSuppressExitAnimation] = createSignal(false);

  const commit = (next: boolean) => {
    if (props.open === undefined) setInternalOpen(next);
    props.onOpenChange?.(next);
  };

  // 用 window.setTimeout/clearTimeout（而不是裸的全局 setTimeout）显式拿到
  // DOM lib 的重载（返回 number），避免和 @types/node 的 NodeJS.Timeout 重载冲突。
  let openTimer: number | undefined;
  let closeTimer: number | undefined;

  const clearTimers = () => {
    if (openTimer !== undefined) {
      window.clearTimeout(openTimer);
      openTimer = undefined;
    }
    if (closeTimer !== undefined) {
      window.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
  };

  const requestOpen = () => {
    clearTimers();
    if (disabled() || open()) return;

    // 组内抢占：如果当前组里已经有一个活跃的 tooltip（哪怕它还在自己的
    // closeDelay 倒计时里，还没真正关闭），claim() 会立即强制关闭它、
    // 把它此刻的矩形要回来——这里直接跳过 openDelay，走"滑入"路径。
    const claimedRect = group?.claim();
    if (claimedRect) {
      setPendingSlideFrom(claimedRect);
      commit(true);
      return;
    }

    openTimer = window.setTimeout(
      () => commit(true),
      props.openDelay ?? provider?.delay() ?? DEFAULT_OPEN_DELAY,
    );
  };

  const requestClose = () => {
    clearTimers();
    closeTimer = window.setTimeout(
      () => commit(false),
      props.closeDelay ?? provider?.closeDelay() ?? DEFAULT_CLOSE_DELAY,
    );
  };

  const openImmediate = () => {
    if (disabled()) return;
    clearTimers();
    commit(true);
  };

  const closeImmediate = () => {
    clearTimers();
    commit(false);
  };

  /**
   * 专门给 <TooltipGroup> 抢占用的关闭方式：和 closeImmediate 的区别是，
   * 这里会额外标记 suppressExitAnimation，告诉 TooltipContent 的 Presence
   * 逻辑跳过退场动画、立即卸载——因为这次关闭是"被新 tooltip 取代"，而不是
   * 用户主动关闭，旧的应该立刻让位，不该和正在滑入的新 tooltip 同时可见。
   * Escape/blur 等正常关闭场景依然走 closeImmediate，退场动画不受影响。
   */
  const forceCloseForGroup = () => {
    setSuppressExitAnimation(true);
    clearTimers();
    commit(false);
  };

  const keepOpen = () => {
    if (closeTimer !== undefined) {
      window.clearTimeout(closeTimer);
      closeTimer = undefined;
    }
  };

  onCleanup(clearTimers);

  // 打开期间把自己登记成"组内当前活跃的 tooltip"，供其他 trigger 打开时
  // claim 走（触发平移过渡）。关闭或卸载时自动取消登记。
  createEffect(() => {
    if (!group || !open()) return;
    const unregister = group.registerActive({
      rect: () => floating()?.getBoundingClientRect(),
      forceClose: forceCloseForGroup,
    });
    onCleanup(unregister);
  });

  const contentId = `tooltip-${createUniqueId()}`;

  const ctx: TooltipContextValue = {
    open,
    disabled,
    contentId,
    reference,
    setReference,
    floating,
    setFloating,
    requestOpen,
    requestClose,
    openImmediate,
    closeImmediate,
    keepOpen,
    pendingSlideFrom,
    clearPendingSlideFrom: () => setPendingSlideFrom(undefined),
    suppressExitAnimation,
    consumeSuppressExitAnimation: () => {
      const value = suppressExitAnimation();
      if (value) setSuppressExitAnimation(false);
      return value;
    },
  };

  return (
    <TooltipContext.Provider value={ctx}>
      {props.children}
    </TooltipContext.Provider>
  );
}
