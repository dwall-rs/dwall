import { createContext, useContext } from "solid-js";

interface TooltipContentContextValue {
  side: "top" | "bottom" | "left" | "right";
  align: "start" | "center" | "end";
}

export const TooltipContentContext =
  createContext<TooltipContentContextValue>();

export function useTooltipContentContext() {
  const ctx = useContext(TooltipContentContext);
  if (!ctx)
    throw new Error(
      "useTooltipContentContext must be used within TooltipContent",
    );
  return ctx;
}
