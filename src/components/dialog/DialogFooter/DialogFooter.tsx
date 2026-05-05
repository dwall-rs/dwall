import { splitProps } from "solid-js";
import type { DialogFooterProps } from "./DialogFooter.types";
import { clsx } from "~/utils";
import { classes } from "./DialogFooter.styles";

export const DialogFooter = (props: DialogFooterProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="dialog-footer"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
