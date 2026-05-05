import { splitProps } from "solid-js";
import type { AlertDialogFooterProps } from "./AlertDialogFooter.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogFooter.styles";

export const AlertDialogFooter = (props: AlertDialogFooterProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="alert-dialog-footer"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
