import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  splitProps,
} from "solid-js";
import { useScrollLock } from "~/hooks/useScrollLock";
import type { DialogProps } from "./Dialog.types";
import { DialogContext } from "./Dialog.context";

export const Dialog = (props: DialogProps) => {
  const [local, others] = splitProps(props, [
    "open",
    "defaultOpen",
    "onOpenChange",
    "lockScroll",
    "dismissOnEscape",
    "children",
  ]);

  const initialOpen = local.open ?? local.defaultOpen ?? false;

  const [show, setShow] = createSignal(initialOpen);

  const [internalOpen, setInternalOpen] = createSignal(
    local.defaultOpen ?? false,
  );

  const open = createMemo(() =>
    local.open === undefined ? internalOpen() : local.open,
  );
  const lockScroll = createMemo(() => props.lockScroll ?? true);

  useScrollLock(() => open() && lockScroll(), {
    allowedSelector: "[data-dialog-surface]",
  });

  const setOpen = (next: boolean) => {
    if (local.open === undefined) {
      setInternalOpen(next);
    }
    local.onOpenChange?.(next);
  };

  createEffect(() => {
    if (!open() || local.dismissOnEscape === false) return;
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || event.defaultPrevented) return;
      event.preventDefault();
      setOpen(false);
    };
    document.addEventListener("keydown", handleKeyDown);
    onCleanup(() => document.removeEventListener("keydown", handleKeyDown));
  });

  createEffect(() => {
    if (open()) {
      setShow(true);
    }
  });

  return (
    <DialogContext.Provider
      value={{
        show,
        setShow,
        open,
        setOpen,
      }}
    >
      <div data-slot="dialog" {...others}>
        {local.children}
      </div>
    </DialogContext.Provider>
  );
};
