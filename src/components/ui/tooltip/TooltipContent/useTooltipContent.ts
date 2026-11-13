import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "~/lib";
import { useTooltipContext } from "../Tooltip/Tooltip.context";
import { createTooltipMiddleware, toPlacement } from "./TooltipContent.utils";
import { usePresence } from "./usePresence";
import type { TooltipContentProps } from "./TooltipContent.types";

export interface UseTooltipContentResult {
  ctx: ReturnType<typeof useTooltipContext>;
  pos: Positioner;

  isVisible: () => boolean;

  mounted: () => boolean;

  animationState: () => "open" | "closed";
  arrowElement: () => Element | undefined;
  setArrowElement: (el: Element) => void;

  contentElement: () => HTMLElement | undefined;
  setContentElement: (el: HTMLElement) => void;

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

  const isVisible = createMemo(
    () => ctx.open() && !pos.middlewareData().hide?.referenceHidden,
  );

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

  const [slideOffset, setSlideOffset] = createSignal<{
    x: number;
    y: number;
  }>();

  const mounted = usePresence({
    visible: isVisible,
    element: contentElement,
    consumeSuppressExitAnimation: () => ctx.consumeSuppressExitAnimation(),
  });

  createEffect(() => {
    if (!mounted()) return;
    const from = ctx.pendingSlideFrom();
    if (!from) return;

    if (!pos.isPositioned()) return;

    const dx = from.x - pos.x();
    const dy = from.y - pos.y();

    setSlideOffset({ x: dx, y: dy });

    ctx.clearPendingSlideFrom();

    requestAnimationFrame(() => setSlideOffset({ x: 0, y: 0 }));
  });

  const slideStyle = createMemo(() => {
    const offset = slideOffset();
    if (!offset) return {};
    const atRest = offset.x === 0 && offset.y === 0;
    return {
      transform: `translate(${offset.x}px, ${offset.y}px)`,

      transition: atRest ? "transform 150ms ease" : undefined,
    };
  });

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
