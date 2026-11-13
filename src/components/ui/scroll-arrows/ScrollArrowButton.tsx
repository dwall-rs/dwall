import { Show } from "solid-js";
import { ChevronDown, ChevronUp } from "lucide-solid";
import { clsx } from "~/utils";
import type { ScrollArrowButtonProps } from "./ScrollArrows.types";

export function ScrollArrowButton(props: ScrollArrowButtonProps) {
  return (
    <Show when={props.visible}>
      <div
        aria-hidden="true"
        data-slot="scroll-arrow"
        data-direction={props.direction}
        class={clsx(
          "pointer-events-none absolute inset-x-0 z-10 flex h-6 select-none items-center justify-center bg-popover text-muted-foreground [&_svg]:size-4",
          props.direction === "down" ? "bottom-0" : "top-0",
          props.class,
        )}
      >
        {props.direction === "down" ? <ChevronDown /> : <ChevronUp />}
      </div>
    </Show>
  );
}
