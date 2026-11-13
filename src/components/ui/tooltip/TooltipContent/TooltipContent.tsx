import { Show, onCleanup, splitProps } from "solid-js";
import { Portal } from "solid-js/web";
import { useTooltipContent } from "./useTooltipContent";
import { TooltipContentContext } from "./TooltipContent.context";
import { getTransformOrigin } from "./TooltipContent.utils";
import { getSide, getAlignment } from "~/lib";
import type { TooltipContentProps } from "./TooltipContent.types";
import { clsx } from "~/utils";

export function TooltipContent(props: TooltipContentProps) {
  const {
    ctx,
    pos,
    mounted,
    animationState,
    setArrowElement,
    setContentElement,
    slideStyle,
  } = useTooltipContent(() => props);

  const [local, rest] = splitProps(props, [
    "side",
    "align",
    "sideOffset",
    "alignOffset",
    "collisionPadding",
    "class",
    "style",
    "children",
  ]);

  return (
    <Show when={mounted()}>
      <Portal>
        <div
          ref={(el) => {
            ctx.setFloating(el);

            onCleanup(() => ctx.setFloating(undefined));
          }}
          data-placement={pos.placement()}
          style={{
            ...pos.floatingStyles(),
            "z-index": 1000,
            opacity: pos.isPositioned() ? 1 : 0,
          }}
        >
          <div style={slideStyle()}>
            <div
              ref={setContentElement}
              id={ctx.contentId}
              role="tooltip"
              data-state={animationState()}
              data-side={getSide(pos.placement())}
              data-align={getAlignment(pos.placement()) ?? "center"}
              onMouseEnter={ctx.keepOpen}
              onMouseLeave={ctx.requestClose}
              style={{
                "transform-origin": getTransformOrigin(pos.placement()),
                ...(typeof local.style === "object" ? local.style : undefined),
              }}
              class={clsx(
                "inline-flex w-fit max-w-xs items-center gap-1.5 rounded-md bg-foreground px-3 py-1.5 text-xs text-background",
                "has-data-[slot=kbd]:pr-1.5",
                "data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95",
                "data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95",

                "data-[side=top]:slide-in-from-bottom-2",
                "data-[side=bottom]:slide-in-from-top-2",
                "data-[side=left]:slide-in-from-right-2",
                "data-[side=right]:slide-in-from-left-2",
                local.class,
              )}
              {...rest}
            >
              <TooltipContentContext.Provider
                value={{
                  middlewareData: pos.middlewareData,
                  reference: ctx.reference,
                  placement: pos.placement,
                  setArrowElement,
                  animationState,
                }}
              >
                {local.children}
              </TooltipContentContext.Provider>
            </div>
          </div>
        </div>
      </Portal>
    </Show>
  );
}
