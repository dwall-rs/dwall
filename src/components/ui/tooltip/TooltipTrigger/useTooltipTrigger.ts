import { onCleanup } from "solid-js";
import { useTooltipContext } from "../Tooltip/Tooltip.context";

export interface UseTooltipTriggerResult {
  ctx: ReturnType<typeof useTooltipContext>;

  attachListeners: (el: Element) => void;
}

export function useTooltipTrigger(): UseTooltipTriggerResult {
  const ctx = useTooltipContext("TooltipTrigger");

  const attachListeners = (el: Element) => {
    let isPointerDown = false;

    const onPointerEnter = (event: Event) => {
      if ((event as PointerEvent).pointerType === "touch") return;
      if (!ctx.disabled()) ctx.requestOpen();
    };
    const onPointerLeave = () => {
      ctx.requestClose();
    };
    const onFocus = () => {
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
