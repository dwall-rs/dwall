import { createMemo, type JSX } from "solid-js";
import { TooltipProviderContext } from "./TooltipProvider.context";
import type {
  TooltipProviderContextValue,
  TooltipProviderProps,
} from "./TooltipProvider.types";

export function TooltipProvider(props: TooltipProviderProps): JSX.Element {
  const delay = createMemo(() => props.delay ?? 0);
  const closeDelay = createMemo(() => props.closeDelay ?? 150);

  const ctx: TooltipProviderContextValue = { delay, closeDelay };

  return (
    <TooltipProviderContext.Provider value={ctx}>
      {props.children}
    </TooltipProviderContext.Provider>
  );
}
