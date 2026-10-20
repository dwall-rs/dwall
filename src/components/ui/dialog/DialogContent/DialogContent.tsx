import { Show, mergeProps, splitProps } from "solid-js";
import type { DialogContentProps } from "./DialogContent.types";
import { clsx } from "~/utils";
import { classes } from "./DialogContent.styles";
import { DialogSurface } from "../DialogSurface";
import { DialogClose } from "../DialogClose";

export const DialogContent = (props: DialogContentProps) => {
  const merged = mergeProps({ showCloseButton: true } as const, props);

  const [local, others] = splitProps(merged, [
    "showCloseButton",
    "class",
    "classList",
    "children",
  ]);

  return (
    <DialogSurface
      {...others}
      role="dialog"
      data-slot="dialog-content"
      class={clsx(classes, local.class)}
      classList={local.classList}
    >
      {local.children}
      <Show when={local.showCloseButton}>
        <DialogClose />
      </Show>
    </DialogSurface>
  );
};
