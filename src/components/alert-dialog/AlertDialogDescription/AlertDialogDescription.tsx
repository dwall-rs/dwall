import { createUniqueId, onMount, splitProps } from "solid-js";
import type { AlertDialogDescriptionProps } from "./AlertDialogDescription.types";
import { clsx } from "~/utils";
import { classes } from "./AlertDialogDescription.styles";
import { useAlertDialogContentContext } from "../AlertDialogContent";

export const AlertDialogDescription = (props: AlertDialogDescriptionProps) => {
  const { setDescriptionID } = useAlertDialogContentContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const id = createUniqueId();

  onMount(() => {
    setDescriptionID(id);
  });

  return (
    <p
      id={id}
      data-slot="alert-dialog-description"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
