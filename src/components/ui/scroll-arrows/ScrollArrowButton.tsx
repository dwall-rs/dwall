import { Show, onCleanup } from "solid-js";
import { ChevronDown, ChevronUp } from "lucide-solid";
import { clsx } from "~/utils";
import type { ScrollArrowButtonProps } from "./ScrollArrows.types";

/**
 * A single scroll-indicator arrow overlaid on the top/bottom edge of a list.
 * While hovered or held it scrolls the target container continuously via
 * requestAnimationFrame, stopping at the boundary (scrollTop no longer changes).
 *
 * The arrow is decorative (`aria-hidden`, not focusable); accessibility is
 * still handled by the list's own listbox/aria-activedescendant semantics.
 */
export function ScrollArrowButton(props: ScrollArrowButtonProps) {
  let frame: number | undefined;

  const stop = () => {
    if (frame !== undefined) {
      cancelAnimationFrame(frame);
      frame = undefined;
    }
  };

  const loop = () => {
    const el = props.target();
    if (!el) {
      stop();
      return;
    }
    // Scroll roughly 1/24 of the height per frame: continuous but not too fast
    const step = Math.max(4, el.clientHeight / 24);
    const before = el.scrollTop;
    el.scrollTop = before + (props.direction === "down" ? step : -step);
    if (el.scrollTop === before) {
      stop();
      return;
    }
    frame = requestAnimationFrame(loop);
  };

  const start = () => {
    stop();
    frame = requestAnimationFrame(loop);
  };

  const attach = (el: HTMLDivElement) => {
    el.addEventListener("pointerenter", start);
    el.addEventListener("pointerdown", start);
    el.addEventListener("pointerleave", stop);
    el.addEventListener("pointerup", stop);
    el.addEventListener("pointercancel", stop);
    onCleanup(() => {
      stop();
      el.removeEventListener("pointerenter", start);
      el.removeEventListener("pointerdown", start);
      el.removeEventListener("pointerleave", stop);
      el.removeEventListener("pointerup", stop);
      el.removeEventListener("pointercancel", stop);
    });
  };

  return (
    <Show when={props.visible}>
      <div
        ref={attach}
        aria-hidden="true"
        data-slot="scroll-arrow"
        data-direction={props.direction}
        class={clsx(
          "absolute inset-x-0 z-10 flex h-6 cursor-default items-center justify-center bg-popover text-muted-foreground select-none [&_svg]:size-4",
          props.direction === "down" ? "bottom-0" : "top-0",
          props.class,
        )}
      >
        {props.direction === "down" ? <ChevronDown /> : <ChevronUp />}
      </div>
    </Show>
  );
}
