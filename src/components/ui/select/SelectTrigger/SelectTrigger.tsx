import type { ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";
import { useSelectTrigger } from "./useSelectTrigger";
import type { SelectTriggerProps } from "./SelectTrigger.types";
import { clsx, mergeRefs } from "~/utils";
import { Button } from "~/components/ui/button";
import { ChevronDown } from "lucide-solid";

export const SelectTrigger = <
  T extends ValidComponent = typeof Button<"button">,
>(
  props: SelectTriggerProps<T>,
) => {
  const { ctx, isDisabled, attachListeners } = useSelectTrigger(() => props);

  return (
    <Dynamic
      {...props}
      disabled={isDisabled()}
      icon={<ChevronDown />}
      iconPosition="right"
      component={(props.component as ValidComponent) ?? Button<"button">}
      variant={props.variant ?? "outline"}
      ref={mergeRefs(ctx.setReference, props.ref, attachListeners)}
      role="combobox"
      aria-haspopup="listbox"
      aria-expanded={ctx.open()}
      aria-controls={ctx.open() ? ctx.contentId : undefined}
      data-state={ctx.open() ? "open" : "closed"}
      data-disabled={isDisabled() ? "" : undefined}
      class={clsx("justify-between", props.class)}
    />
  );
};
