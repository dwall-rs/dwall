import { onCleanup } from "solid-js";
import { useTooltipContext } from "../Tooltip/Tooltip.context";

export interface UseTooltipTriggerResult {
  ctx: ReturnType<typeof useTooltipContext>;
  /**
   * Binds the open/close native event listeners to the real DOM element (called via ref).
   * Uses addEventListener rather than JSX onXxx props so as not to occupy those prop
   * names — callers can freely pass their own native event handlers to
   * <TooltipTrigger onClick={...} onMouseEnter={...}>; the two are entirely separate
   * mechanisms (one goes through Solid's event system, the other is native
   * addEventListener) and neither overrides the other — both fire normally.
   */
  attachListeners: (el: Element) => void;
}

/**
 * Encapsulates all the event logic TooltipTrigger needs to bind:
 * - pointerenter/pointerleave (hover in/out) go through openDelay/closeDelay
 * - focus/blur (keyboard navigation) skip the delay and toggle immediately — this is
 *   an accessibility requirement; keyboard users shouldn't be forced to wait
 * - Escape closes immediately and stops propagation (so it isn't handled again by
 *   other outer logic also listening for Escape)
 * - mousedown closes immediately — close to shadcn/Radix behavior: if the user starts
 *   clicking the element, they intend to interact with it, and a tooltip still sitting
 *   there would block the view, so it should get out of the way at once without
 *   waiting for pointerleave or closeDelay.
 *
 * There's an easy-to-miss pitfall between mousedown and focus: the native event order
 * is mousedown → focus → mouseup → click — when clicking a focusable element (e.g.
 * <button>), the browser focuses it automatically after mousedown. Without handling,
 * mousedown just closed the tooltip and the browser-triggered focus immediately
 * reopens it, with the two logics fighting each other. The isPointerDown flag marks
 * "was this focus passively triggered right after a click"; if so, skip reopening,
 * and only a focus triggered by real keyboard Tab navigation opens the tooltip.
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
      // This focus immediately following mousedown is an automatic browser side
      // effect, not an indication that the user is navigating with the keyboard, so
      // it shouldn't reopen the tooltip that mousedown just closed.
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
    // The flag must be cleared on mouseup, and we deliberately listen on document
    // rather than the element's own mouseup — if the mouse is pressed and dragged
    // outside the element before release, the element never receives mouseup, the
    // flag stays stuck at true, and subsequent genuine keyboard focus is wrongly skipped.
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
