import type { ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";
import { Button } from "~/components/ui/button";
import { useDialogTrigger } from "~/components/ui/dialog";
import { mergeRefs } from "~/utils";
import type { AlertDialogTriggerProps } from "./AlertDialogTrigger.types";

/**
 * 复用 Dialog 的 trigger 逻辑（多态、disabled、aria-haspopup/expanded/state、
 * addEventListener 不覆盖用户 onClick），只把 data-slot 换成 alert 的。
 */
export const AlertDialogTrigger = <
  T extends ValidComponent = typeof Button<"button">,
>(
  props: AlertDialogTriggerProps<T>,
) => {
  const { open, isDisabled, attachListeners } = useDialogTrigger(() => props);

  return (
    <Dynamic
      {...props}
      component={(props.component as ValidComponent) ?? Button<"button">}
      ref={mergeRefs(props.ref, attachListeners)}
      disabled={isDisabled()}
      data-slot="alert-dialog-trigger"
      aria-haspopup="dialog"
      aria-expanded={open()}
      data-state={open() ? "open" : "closed"}
    />
  );
};
