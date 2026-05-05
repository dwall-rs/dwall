import {
  createMemo,
  createSignal,
  mergeProps,
  Show,
  splitProps,
} from "solid-js";
import { useTooltipContext } from "../Tooltip/Tooltip.context";
import { Portal } from "solid-js/web";
import type { TooltipContentProps } from "./TooltipContent.types";
import { clsx } from "~/utils";
import { TooltipContentContext } from "./TooltipContent.context";
import { useTooltipContent } from "./useTooltipContent";

export const TooltipContent = (props: TooltipContentProps) => {
  const ctx = useTooltipContext();

  const merged = mergeProps(
    { side: "top", align: "center", alignOffset: 0, sideOffset: 4 } as const,
    props,
  );
  const [local, others] = splitProps(merged, [
    "side",
    "align",
    "alignOffset",
    "sideOffset",
    "class",
    "classList",
    "children",
  ]);

  const [ref, setRef] = createSignal<HTMLDivElement>();

  const { shouldRender, position } = useTooltipContent(ref, () => local);

  const isOpen = createMemo(() => ctx.open());

  return (
    <Show when={shouldRender()}>
      <Portal>
        <div
          ref={setRef}
          data-slot="tooltip-content"
          data-open={isOpen() || undefined}
          data-closed={!isOpen() || undefined}
          data-align={position.align}
          data-side={position.side}
          id={ctx.contentId}
          role="tooltip"
          class={clsx(
            "fixed z-50 pointer-events-none inline-flex w-fit max-w-xs origin-(--transform-origin) items-center gap-1.5 rounded-md bg-neutral-950 dark:bg-neutral-50 px-3 py-1.5 text-xs text-white dark:text-neutral-950 has-data-[slot=kbd]:pr-1.5 data-[side=bottom]:slide-in-from-top-2 data-[side=inline-end]:slide-in-from-left-2 data-[side=inline-start]:slide-in-from-right-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2 **:data-[slot=kbd]:relative **:data-[slot=kbd]:isolate **:data-[slot=kbd]:z-50 **:data-[slot=kbd]:rounded-sm data-[state=delayed-open]:animate-in data-[state=delayed-open]:fade-in-0 data-[state=delayed-open]:zoom-in-95 data-open:animate-in data-open:fade-in-0 data-open:zoom-in-95 data-closed:animate-out data-closed:fade-out-0 data-closed:zoom-out-95",
            local.class,
          )}
          style={{
            top: `${position.top}px`,
            left: `${position.left}px`,
            visibility: position.top === 0 ? "hidden" : "visible",
          }}
          {...others}
        >
          <TooltipContentContext.Provider
            value={{
              get side() {
                return position.side;
              },
              get align() {
                return position.align;
              },
            }}
          >
            {local.children}
          </TooltipContentContext.Provider>
        </div>
      </Portal>
    </Show>
  );
};
