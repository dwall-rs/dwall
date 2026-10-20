import { mergeProps, splitProps } from "solid-js";
import type { AlertDialogContentProps } from "./AlertDialogContent.types";
import { clsx } from "~/utils";
import { DialogSurface } from "~/components/ui/dialog";
import classes from "./AlertDialogContent.styles";

export const AlertDialogContent = (props: AlertDialogContentProps) => {
  const merged = mergeProps({ size: "default" } as const, props);

  const [local, others] = splitProps(merged, ["size", "class", "classList"]);

  return (
    <DialogSurface
      {...others}
      role="alertdialog"
      dismissOnOverlayClick={false}
      data-slot="alert-dialog-content"
      data-size={local.size}
      class={clsx(classes, local.class)}
      classList={local.classList}
    />
  );
};
