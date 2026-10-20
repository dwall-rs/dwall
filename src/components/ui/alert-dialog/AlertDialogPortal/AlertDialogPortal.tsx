import type { AlertDialogPortalProps } from "./AlertDialogPortal.types";
import { DialogPortal } from "~/components/ui/dialog";

export const AlertDialogPortal = (props: AlertDialogPortalProps) => (
  <DialogPortal {...props} data-slot="alert-dialog-portal" />
);
