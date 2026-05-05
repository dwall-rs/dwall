import { createContext, useContext } from "solid-js";

type TooltipContextValue = {
  open: () => boolean;
  setOpen: (v: boolean) => void;
  triggerRef: HTMLElement | undefined;
  setTriggerRef: (el: HTMLElement) => void;
  contentId: string;
  openDelay: number;
  closeDelay: number;
};

export const TooltipContext = createContext<TooltipContextValue>();

export function useTooltipContext() {
  const ctx = useContext(TooltipContext);
  if (!ctx)
    throw new Error("Tooltip components must be used within Tooltip.Root");
  return ctx;
}
