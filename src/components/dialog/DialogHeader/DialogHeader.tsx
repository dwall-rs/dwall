import { splitProps } from "solid-js";
import type { DialogHeaderProps } from "./DialogHeader.types";
import { clsx } from "~/utils";
import { classes } from "./DialogHeader.styles";

export const DialogHeader = (props: DialogHeaderProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="dialog-header"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
