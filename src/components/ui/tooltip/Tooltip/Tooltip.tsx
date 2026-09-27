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
 * Tooltip root component: renders no DOM itself, only managing state and
 * providing context. Actual rendering is delegated to
 * <TooltipTrigger>/<TooltipContent>/<TooltipArrow>.
 *
 * When wrapped in <TooltipGroup>, switching hover between adjacent triggers in
 * the group skips openDelay and triggers a "slide from the old position to the
 * new position" transition (see TooltipGroup.context.ts for details).
 *
 * @example
 * ```tsx
 * <Tooltip openDelay={300}>
 *   <TooltipTrigger>Hover me</TooltipTrigger>
 *   <TooltipContent>
 *     Some tooltip text
 *     <TooltipArrow />
 *   </TooltipContent>
 * </Tooltip>
 * ```
 */
export function Tooltip(props: TooltipProps): JSX.Element {
  const group = useTooltipGroupContext(); // undefined is normal: no TooltipGroup used
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

  // Use window.setTimeout/clearTimeout (not the bare global setTimeout) to get
  // the DOM lib overload (number), avoiding @types/node's NodeJS.Timeout clash.
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

    // Group preemption: if the group already has an active tooltip (even one
    // still in its closeDelay countdown, not yet closed), claim() force-closes
    // it and takes its current rect — here we skip openDelay and "slide in".
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
   * Close variant for <TooltipGroup> preemption: it also sets
   * suppressExitAnimation, telling TooltipContent's Presence logic to skip the
   * exit animation and unmount immediately — this close is "replaced by a new
   * tooltip", not user-initiated, so the old one must yield instead of staying
   * visible beside the new one sliding in. Escape/blur still use
   * closeImmediate; their exit animation is unaffected.
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

  // While open, register as the "currently active tooltip in the group" so
  // other triggers can claim it on open (triggering the slide transition).
  // Automatically unregistered on close or unmount.
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
