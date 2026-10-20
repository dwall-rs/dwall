import { createUniqueId, onMount, splitProps } from "solid-js";
import type { AlertDialogDescriptionProps } from "./AlertDialogDescription.types";
import { clsx } from "~/utils";
import classes from "./AlertDialogDescription.styles";
import { useDialogContentContext } from "~/components/ui/dialog";

export const AlertDialogDescription = (props: AlertDialogDescriptionProps) => {
  const { setDescriptionID } = useDialogContentContext();

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
      classList={local.classList}
      {...others}
    />
  );
};
