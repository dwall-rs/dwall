import { createEffect, createMemo, createSignal, onCleanup } from "solid-js";
import {
  createPositioner,
  toPlacement,
  type Placement,
  type Positioner,
} from "~/lib";
import { useSelectContext } from "../Select/Select.context";
import { createSelectMiddleware } from "./SelectContent.utils";
import type { SelectContentProps } from "./SelectContent.types";

export interface UseSelectContentResult {
  ctx: ReturnType<typeof useSelectContext>;
  pos: Positioner;

  placement: () => Placement;

  aligned: () => boolean;

  maxHeight: () => number | undefined;

  isVisible: () => boolean;

  mounted: () => boolean;

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

  const alignItemWithTrigger = () => props().alignItemWithTrigger ?? true;

  const useItemAlign = createMemo(
    () => alignItemWithTrigger() && ctx.value() != null,
  );

  const placement = createMemo<Placement>(() => {
    const legacy = props().placement;
    if (legacy) return legacy;

    return toPlacement(props().side ?? "bottom", props().align ?? "start");
  });

  const pos = createPositioner(ctx.reference, ctx.floating, {
    placement,
    strategy: "fixed",
    middleware: () =>
      createSelectMiddleware({
        useItemAlign: useItemAlign(),
        selectedValue: ctx.value,
        scrollElement: contentElement,
        sideOffset: props().sideOffset ?? 4,
        alignOffset: props().alignOffset ?? 0,
        collisionPadding: props().collisionPadding ?? 8,
        onAvailableHeightChange: (availableHeight) => {
          setFallbackMaxHeight(Math.max(availableHeight, 120));
        },
      }),
  });

  const aligned = useItemAlign;

  const maxHeight = createMemo(() => {
    if (useItemAlign()) {
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

  createEffect(() => {
    if (ctx.open()) {
      pos.update();
    }
  });

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

  const mounted = isVisible;

  return {
    ctx,
    pos,
    placement,
    aligned,
    maxHeight,
    isVisible,
    mounted,
    animationState,
    contentElement,
    setContentElement,
  };
}
