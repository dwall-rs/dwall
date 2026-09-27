import { createContext, useContext } from "solid-js";
import type { TooltipProviderContextValue } from "./TooltipProvider.types";

export const TooltipProviderContext =
  createContext<TooltipProviderContextValue>();

export function useTooltipProviderContext():
  | TooltipProviderContextValue
  | undefined {
  return useContext(TooltipProviderContext);
}
