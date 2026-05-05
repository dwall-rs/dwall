import { createSignal, createUniqueId, type ParentProps } from "solid-js";
import { TooltipContext } from "./Tooltip.context";

export const Tooltip = (
  props: ParentProps & { openDelay?: number; closeDelay?: number },
) => {
  let triggerRef: HTMLElement | undefined;

  const [open, setOpen] = createSignal(false);

  const setTriggerRef = (el: HTMLElement) => {
    triggerRef = el;
  };

  const contentId = createUniqueId();

  const contextValue = {
    open,
    setOpen,
    get triggerRef() {
      return triggerRef;
    },
    setTriggerRef,
    contentId,
    openDelay: props.openDelay ?? 100,
    closeDelay: props.closeDelay ?? 100,
  };

  return (
    <TooltipContext.Provider value={contextValue}>
      {props.children}
    </TooltipContext.Provider>
  );
};
