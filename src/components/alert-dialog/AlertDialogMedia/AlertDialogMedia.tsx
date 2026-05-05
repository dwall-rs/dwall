import { splitProps } from "solid-js";
import type { AlertDialogMediaProps } from "./AlertDialogMedia.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogMedia.styles";

export const AlertDialogMedia = (props: AlertDialogMediaProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="alert-dialog-media"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
