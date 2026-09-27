import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "~/lib";
import { useSelectContext } from "../Select/Select.context";
import { createSelectMiddleware } from "./SelectContent.utils";
import type { SelectContentProps } from "./SelectContent.types";

export interface UseSelectContentResult {
  ctx: ReturnType<typeof useSelectContext>;
  pos: Positioner;
  /**
   * The max height the panel should use: with a selected value it reads the
   * panelHeight that alignSelectedItem computed itself (bypassing the size
   * middleware); without a value (placeholder state, edge-anchored dropdown)
   * it uses the available space computed by the size middleware. The two
   * scenarios use different height sources, merged here into a single public
   * value so the presentation layer does not care which one was taken.
   */
  maxHeight: () => number | undefined;
  /** Combines the open state with the hide middleware's referenceHidden check */
  isVisible: () => boolean;
  /**
   * Compatibility field: SelectContent no longer unmounts its Portal on
   * close (to preserve the ctx.items registration), so mounted is no longer
   * used by the presentation layer to drive <Show>. It is currently
   * equivalent to isVisible and is kept only to avoid breaking the exported
   * useSelectContent return type.
   */
  mounted: () => boolean;
  /** State driving data-state; on open it lags one frame behind isVisible,
   * giving positioning strategies like "selected-item alignment" time — they
   * must read the child elements (options) first to compute accurately. */
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

  // SelectContent also does not unmount on close (to preserve the ctx.items
  // registration), so createPositioner is not recreated and does not
  // auto-compute a position on every open like Tooltip/Popover. Recompute
  // explicitly on each open to fix "after the trigger moved due to an
  // external layout change on its first open, the panel still sits at the old
  // coordinates on the second open".
  createEffect(() => {
    if (ctx.open()) {
      pos.update();
    }
  });

  // --- Delay one frame before triggering the animation class on open ---
  // The "selected-item alignment" strategy must read the child elements (the
  // <li data-value> elements SelectItem renders) to compute the correct
  // position. If the animation starts playing before that positioning
  // finishes, the panel only "jumps" to the correct position halfway through
  // the animation — wait one frame to give the positioning calculation time
  // (same idea as how production-tooltip handles arrow position).
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

  // SelectContent no longer unmounts its children on close (otherwise
  // ctx.items would be cleared and SelectValue could not read the labels), so
  // mounted no longer carries any "deferred/immediate unmount" semantics and
  // simply stays in sync with isVisible. The field is kept only for
  // compatibility with the exported useSelectContent return type; new
  // presentation-layer logic should always use isVisible.
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
