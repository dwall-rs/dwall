import { createUniqueId, onMount, splitProps } from "solid-js";
import type { AlertDialogTitleProps } from "./AlertDialogTitle.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogTitle.styles";
import { useAlertDialogContentContext } from "../AlertDialogContent";

export const AlertDialogTitle = (props: AlertDialogTitleProps) => {
  const { setTitleID } = useAlertDialogContentContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const id = createUniqueId();

  onMount(() => {
    setTitleID(id);
  });

  return (
    <h2
      id={id}
      data-slot="alert-dialog-title"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
