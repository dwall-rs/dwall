import { createUniqueId, onMount, splitProps } from "solid-js";
import type { DialogDescriptionProps } from "./DialogDescription.types";
import { clsx } from "~/utils";
import { classes } from "./DialogDescription.styles";
import { useDialogContentContext } from "../DialogContent";

export const DialogDescription = (props: DialogDescriptionProps) => {
  const { setDescriptionID } = useDialogContentContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const id = createUniqueId();

  onMount(() => {
    setDescriptionID(id);
  });

  return (
    <p
      id={id}
      data-slot="dialog-description"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
