import type { ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";

import { useTooltipTrigger } from "./useTooltipTrigger";
import type { TooltipTriggerProps } from "./TooltipTrigger.types";

import { Button } from "~/components/ui/button";
import { mergeRefs } from "~/utils";

export const TooltipTrigger = <
  T extends ValidComponent = typeof Button<"button">,
>(
  props: TooltipTriggerProps<T>,
) => {
  const { ctx, attachListeners } = useTooltipTrigger();

  return (
    <Dynamic
      {...props}
      component={(props.component as ValidComponent) ?? Button<"button">}
      ref={mergeRefs(ctx.setReference, props.ref, attachListeners)}
      aria-describedby={ctx.open() ? ctx.contentId : undefined}
      data-state={ctx.open() ? "open" : "closed"}
      // data-disabled={props.disabled ? "" : undefined}
    />
  );
};
