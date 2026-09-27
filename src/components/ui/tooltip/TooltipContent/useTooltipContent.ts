import { createSignal, createMemo, createEffect, onCleanup } from "solid-js";
import { createPositioner, type Positioner } from "~/lib";
import { useTooltipContext } from "../Tooltip/Tooltip.context";
import { createTooltipMiddleware, toPlacement } from "./TooltipContent.utils";
import type { TooltipContentProps } from "./TooltipContent.types";

/** Fallback unmount delay (ms) when no CSS animation fires — e.g. without
 * tailwindcss-animate, or when custom styles use transitions instead of
 * @keyframes so animationend never fires, leaving the element stuck in the DOM. */
const EXIT_FALLBACK_MS = 300;

export interface UseTooltipContentResult {
  ctx: ReturnType<typeof useTooltipContext>;
  pos: Positioner;
  /** Combines the open state + the hide middleware's referenceHidden flag: "should it logically be open" */
  isVisible: () => boolean;
  /**
   * Whether the node should stay in the DOM. Once isVisible turns false,
   * mounted does not follow right away — it waits for the exit animation (or
   * the fallback timeout). Drive <Show> with it rather than isVisible,
   * otherwise the element unmounts before the exit animation can play.
   */
  mounted: () => boolean;
  /**
   * The state that actually drives data-state (and the CSS enter/exit
   * classes); the view layer should use it, not isVisible.
   *
   * It differs from isVisible only on the "open" side: the moment isVisible
   * turns true, animationState does not switch to "open" right away — it
   * waits a frame (requestAnimationFrame). That gap lets the arrow
   * (TooltipArrow) mount, register with the arrow middleware, and get the
   * right position offset. Without it the animation would start before the
   * arrow's position is computed, and the arrow would "jump" into place
   * halfway through — looking like the arrow lags a beat behind the content.
   * That is because the content's fade/scale is CSS-driven (a parent's
   * opacity/transform reaches all children, including the arrow, so both
   * should be in sync) while the arrow's exact position comes from JS
   * computed left/top — two independent mechanisms that only look consistent
   * when "animation starts" after "arrow position computed". Closing needs
   * no delay: when isVisible turns false, animationState switches at once.
   */
  animationState: () => "open" | "closed";
  arrowElement: () => Element | undefined;
  setArrowElement: (el: Element) => void;
  /** The inner content element (the one actually playing the enter/exit animation), which TooltipContent must ref */
  contentElement: () => HTMLElement | undefined;
  setContentElement: (el: HTMLElement) => void;
  /**
   * Style applied to the "slide layer" (between the positioning and content
   * layers). On a TooltipGroup preemption it starts non-zero, then a CSS
   * transition smoothly zeroes it next frame, producing the "slide from the
   * old tooltip's position to the new one" effect; normally an empty object.
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

  // When the reference scrolls entirely out of the viewport (e.g. its own
  // container is scrolled), hide flags referenceHidden; hide the floating
  // layer accordingly so it doesn't hover next to an unseen trigger.
  const isVisible = createMemo(
    () => ctx.open() && !pos.middlewareData().hide?.referenceHidden,
  );

  // --- Arrow and content enter in sync: delay one frame before firing the
  // animation classes ---
  // See the animationState comment: only when "animation start" comes after
  // "arrow position computed" can the arrow avoid jumping mid-animation.
  const [animationState, setAnimationState] = createSignal<"open" | "closed">(
    "closed",
  );

  createEffect(() => {
    if (isVisible()) {
      const raf = requestAnimationFrame(() => setAnimationState("open"));
      onCleanup(() => cancelAnimationFrame(raf));
    } else {
      // Closing needs no wait — the exit animation fades/shrinks outward from
      // the settled position, so the arrow's position is a non-issue: no delay.
      setAnimationState("closed");
    }
  });

  // --- "Slide-in" transition on TooltipGroup preemption (FLIP) ---
  const [slideOffset, setSlideOffset] = createSignal<{
    x: number;
    y: number;
  }>();

  const [mounted, setMounted] = createSignal(isVisible());

  createEffect(() => {
    if (!mounted()) return;
    const from = ctx.pendingSlideFrom();
    if (!from) return;
    // Must wait for real positioning (final coordinates known) to compute the slide delta.
    if (!pos.isPositioned()) return;

    const dx = from.x - pos.x();
    const dy = from.y - pos.y();
    // Step 1: set a non-zero offset — stacked with floatingStyles()' final
    // coordinates, the new tooltip appears exactly where the old one was,
    // with no jump.
    setSlideOffset({ x: dx, y: dy });
    // Clear right after consuming it once, so a later autoUpdate
    // (reposition on scroll/resize) doesn't re-apply this offset.
    ctx.clearPendingSlideFrom();

    // Step 2: zero the offset on the next frame. This update carries a
    // transition (see slideStyle below), so the browser animates from the
    // non-zero offset back to 0 — sliding from the old position to the new.
    // requestAnimationFrame (not synchronous zeroing) ensures the browser
    // paints frame one first (non-zero offset, no transition), giving the
    // next change a start point instead of merging it into a single jump.
    requestAnimationFrame(() => setSlideOffset({ x: 0, y: 0 }));
  });

  const slideStyle = createMemo(() => {
    const offset = slideOffset();
    if (!offset) return {};
    const atRest = offset.x === 0 && offset.y === 0;
    return {
      transform: `translate(${offset.x}px, ${offset.y}px)`,
      // Only the "zero it out" step carries a transition: the non-zero frame must
      // snap into place with no animation, otherwise you'd first see it fly in from
      // the viewport origin instead of from the old tooltip's position.
      transition: atRest ? "transform 150ms ease" : undefined,
    };
  });

  // --- Exit animation support (Presence) ---
  // The moment isVisible flips true → false, don't let <Show> unmount the node right
  // away: first switch data-state to closed (triggering the exit-animation CSS
  // class), then wait for this node's own animationend (or the fallback timeout)
  // before actually setting mounted to false and handing unmounting to <Show>.
  createEffect((wasVisible: boolean) => {
    const visible = isVisible();
    const el = contentElement();

    if (visible) {
      setMounted(true);
    } else if (wasVisible && el) {
      if (ctx.consumeSuppressExitAnimation()) {
        // Force-closed after being preempted by a TooltipGroup: skip the exit
        // animation and unmount immediately, so it doesn't coexist on screen with the
        // new tooltip sliding in (this is the root cause of the "two tooltips at once"
        // bug — previously every close, whatever the reason, went through the exit
        // animation path).
        setMounted(false);
      } else {
        const handleAnimationEnd = (e: AnimationEvent) => {
          // Ignore animationend events bubbling up from children; only honor the animation played by the content element itself
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
