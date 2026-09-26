import { Show, onCleanup } from "solid-js";
import { ChevronDown, ChevronUp } from "lucide-solid";
import { clsx } from "~/utils";
import type { ScrollArrowButtonProps } from "./ScrollArrows.types";

/**
 * 单个滚动提示箭头：覆盖在列表上/下沿，悬停或按住时用 requestAnimationFrame
 * 持续滚动目标容器，到边界后 scrollTop 不再变化即自动停止。
 *
 * 箭头是装饰元素(`aria-hidden`、不可聚焦)，无障碍仍由列表自身的
 * listbox/aria-activedescendant 承担。
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
    // 每帧约滚动 1/24 高度，连续但不至于过快
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
