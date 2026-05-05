import { splitProps } from "solid-js";
import type { AlertDialogOverlayProps } from "./AlertDialogOverlay.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogOverlay.styles";
import { useAlertDialogContext } from "../AlertDialog";

export const AlertDialogOverlay = (props: AlertDialogOverlayProps) => {
  const { open, setShow } = useAlertDialogContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="alert-dialog-overlay"
      role="presentation"
      aria-hidden="true"
      data-open={open()}
      class={clsx(classes, local.class)}
      onAnimationEnd={() => {
        if (open()) return;
        setShow(false);
      }}
      {...others}
    />
  );
};
