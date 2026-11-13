import { createContext, useContext } from "solid-js";
import type { Rect } from "~/lib";

export interface ActiveTooltipEntry {
  rect: () => Rect | undefined;

  forceClose: () => void;
}

export interface TooltipGroupContextValue {
  registerActive: (entry: ActiveTooltipEntry) => () => void;

  claim: () => Rect | undefined;
}

export const TooltipGroupContext = createContext<TooltipGroupContextValue>();

export function useTooltipGroupContext(): TooltipGroupContextValue | undefined {
  return useContext(TooltipGroupContext);
}
