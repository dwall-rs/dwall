import type { AlertDialogActionProps } from "./AlertDialogAction.types";
import { Button } from "~/components/ui/button";
import { useDialogContext } from "~/components/ui/dialog";

export const AlertDialogAction = (props: AlertDialogActionProps) => {
  const { setOpen } = useDialogContext();

  return (
    <Button
      {...props}
      data-slot="alert-dialog-action"
      onClick={(event) => {
        props.onClick?.(event);
        setOpen(false);
      }}
    />
  );
};
