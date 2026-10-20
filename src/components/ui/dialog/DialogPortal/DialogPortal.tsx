import type { DialogPortalProps } from "./DialogPortal.types";
import { Portal } from "solid-js/web";

export const DialogPortal = (props: DialogPortalProps) => {
  return (
    <Portal>
      <div data-slot="dialog-portal" {...props} />
    </Portal>
  );
};
