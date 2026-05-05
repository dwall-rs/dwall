import { splitProps } from "solid-js";
import type { DialogOverlayProps } from "./DialogOverlay.types";
import { clsx } from "~/utils";
import { classes } from "./DialogOverlay.styles";
import { useDialogContext } from "../Dialog";

export const DialogOverlay = (props: DialogOverlayProps) => {
  const { open, setShow, setOpen } = useDialogContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="dialog-overlay"
      role="presentation"
      aria-hidden="true"
      data-open={open()}
      class={clsx(classes, local.class)}
      onClick={() => setOpen(false)}
      onAnimationEnd={() => {
        if (open()) return;
        setShow(false);
      }}
      {...others}
    />
  );
};
