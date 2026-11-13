import { TooltipGroupContext } from "./TooltipGroup.context";
import type {
  ActiveTooltipEntry,
  TooltipGroupContextValue,
} from "./TooltipGroup.context";
import type { TooltipGroupProps } from "./TooltipGroup.types";

export function TooltipGroup(props: TooltipGroupProps) {
  let activeEntry: ActiveTooltipEntry | undefined;

  const registerActive: TooltipGroupContextValue["registerActive"] = (
    entry,
  ) => {
    activeEntry = entry;
    return () => {
      if (activeEntry === entry) activeEntry = undefined;
    };
  };

  const claim: TooltipGroupContextValue["claim"] = () => {
    if (!activeEntry) return undefined;
    const rect = activeEntry.rect();
    activeEntry.forceClose();
    activeEntry = undefined;

    if (!rect || (rect.width === 0 && rect.height === 0)) return undefined;
    return rect;
  };

  const ctx: TooltipGroupContextValue = { registerActive, claim };

  return (
    <TooltipGroupContext.Provider value={ctx}>
      {props.children}
    </TooltipGroupContext.Provider>
  );
}
