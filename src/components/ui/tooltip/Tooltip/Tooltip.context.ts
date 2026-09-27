import { createContext, useContext } from "solid-js";
import type { TooltipContextValue } from "./Tooltip.types";

export const TooltipContext = createContext<TooltipContextValue>();

export function useTooltipContext(component: string): TooltipContextValue {
  const ctx = useContext(TooltipContext);
  if (!ctx) {
    throw new Error(`<${component}> must be rendered inside <Tooltip>`);
  }
  return ctx;
}
