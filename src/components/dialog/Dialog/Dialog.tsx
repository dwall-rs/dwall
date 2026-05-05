import { createSignal } from "solid-js";
import type { DialogProps } from "./Dialog.types";
import { DialogContext } from "./Dialog.context";

export const Dialog = (props: DialogProps) => {
  // 实际显示或卸载信号
  const [show, setShow] = createSignal(false);
  // 动画控制信号
  const [open, setOpen] = createSignal(false);

  return (
    <DialogContext.Provider
      value={{
        show,
        setShow,
        open,
        setOpen,
      }}
    >
      <div data-slot="dialog" role="alert" {...props} />
    </DialogContext.Provider>
  );
};
