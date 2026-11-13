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

export function Tooltip(props: TooltipProps): JSX.Element {
  const group = useTooltipGroupContext();
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
