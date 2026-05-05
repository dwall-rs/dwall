import { createUniqueId, onMount, splitProps } from "solid-js";
import type { DialogTitleProps } from "./DialogTitle.types";
import { clsx } from "~/utils";
import { classes } from "./DialogTitle.styles";
import { useDialogContentContext } from "../DialogContent";

export const DialogTitle = (props: DialogTitleProps) => {
  const { setTitleID } = useDialogContentContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const id = createUniqueId();

  onMount(() => {
    setTitleID(id);
  });

  return (
    <h2
      id={id}
      data-slot="dialog-title"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
