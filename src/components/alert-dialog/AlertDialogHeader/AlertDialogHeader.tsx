import { splitProps } from "solid-js";
import type { AlertDialogHeaderProps } from "./AlertDialogHeader.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogHeader.styles";

export const AlertDialogHeader = (props: AlertDialogHeaderProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="alert-dialog-header"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
