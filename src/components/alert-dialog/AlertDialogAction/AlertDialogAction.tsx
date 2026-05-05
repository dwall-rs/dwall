import type { AlertDialogActionProps } from "./AlertDialogAction.types";
import { Button } from "~/components/button";

export const AlertDialogAction = (props: AlertDialogActionProps) => {
  return <Button data-slot="alert-dialog-action" {...props} />;
};
