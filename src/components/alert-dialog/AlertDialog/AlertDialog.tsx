import { createSignal } from "solid-js";
import type { AlertDialogProps } from "./AlertDialog.types";
import { AlertDialogContext } from "./AlertDialog.context";

export const AlertDialog = (props: AlertDialogProps) => {
  // 实际显示或卸载信号
  const [show, setShow] = createSignal(false);
  // 动画控制信号
  const [open, setOpen] = createSignal(false);

  return (
    <AlertDialogContext.Provider
      value={{
        show,
        setShow,
        open,
        setOpen,
      }}
    >
      <div data-slot="alert-dialog" role="alert" {...props} />
    </AlertDialogContext.Provider>
  );
};
