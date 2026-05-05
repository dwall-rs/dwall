import { mergeProps } from "solid-js";
import type { AlertDialogCancelProps } from "./AlertDialogCancel.types";
import { Button } from "~/components/button";
import { useAlertDialogContext } from "../AlertDialog";

export const AlertDialogCancel = (props: AlertDialogCancelProps) => {
  const { setOpen } = useAlertDialogContext();

  const merged = mergeProps({ variant: "outline", size: "md" } as const, props);

  return (
    <Button
      {...merged}
      data-slot="alert-dialog-cancel"
      onClick={() => setOpen(false)}
    />
  );
};
