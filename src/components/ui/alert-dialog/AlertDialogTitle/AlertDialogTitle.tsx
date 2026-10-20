import { createUniqueId, onMount, splitProps } from "solid-js";
import type { AlertDialogTitleProps } from "./AlertDialogTitle.types";
import { clsx } from "~/utils";
import classes from "./AlertDialogTitle.styles";
import { useDialogContentContext } from "~/components/ui/dialog";

export const AlertDialogTitle = (props: AlertDialogTitleProps) => {
  const { setTitleID } = useDialogContentContext();

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
      classList={local.classList}
      {...others}
    />
  );
};
