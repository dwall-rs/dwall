import type { AlertDialogPortalProps } from "./AlertDialogPortal.types";
import { Portal } from "solid-js/web";

export const AlertDialogPortal = (props: AlertDialogPortalProps) => {
  return (
    <Portal>
      <div data-slot="alert-dialog-portal" {...props} />
    </Portal>
  );
};
