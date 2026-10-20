import type { ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";

import { useDialogTrigger } from "./useDialogTrigger";
import type { DialogTriggerProps } from "./DialogTrigger.types";

import { Button } from "~/components/ui/button";
import { mergeRefs } from "~/utils";

export const DialogTrigger = <
  T extends ValidComponent = typeof Button<"button">,
>(
  props: DialogTriggerProps<T>,
) => {
  const { open, isDisabled, attachListeners } = useDialogTrigger(() => props);

  return (
    <Dynamic
      {...props}
      component={(props.component as ValidComponent) ?? Button<"button">}
      ref={mergeRefs(props.ref, attachListeners)}
      disabled={isDisabled()}
      data-slot="dialog-trigger"
      aria-haspopup="dialog"
      aria-expanded={open()}
      data-state={open() ? "open" : "closed"}
    />
  );
};
