import { TooltipGroupContext } from "./TooltipGroup.context";
import type {
  ActiveTooltipEntry,
  TooltipGroupContextValue,
} from "./TooltipGroup.context";
import type { TooltipGroupProps } from "./TooltipGroup.types";

export function TooltipGroup(props: TooltipGroupProps) {
  // Deliberately store the "currently active entry" in a plain mutable variable
  // rather than a Solid signal: this is purely internal state for coordination
  // between components and doesn't need to drive any rendering, so a signal would
  // only add unnecessary reactive overhead and redundant re-render triggers.
  let activeEntry: ActiveTooltipEntry | undefined;

  const registerActive: TooltipGroupContextValue["registerActive"] = (
    entry,
  ) => {
    activeEntry = entry;
    return () => {
      // Only clear the entry this call registered, so that when "A hasn't
      // unregistered yet but B has already claimed", A's cleanup doesn't wipe the
      // activeEntry that B just set.
      if (activeEntry === entry) activeEntry = undefined;
    };
  };

  const claim: TooltipGroupContextValue["claim"] = () => {
    if (!activeEntry) return undefined;
    const rect = activeEntry.rect();
    activeEntry.forceClose();
    activeEntry = undefined;

    // Defensive check: a genuinely visible tooltip can't have both width and height
    // zero — such an all-zero rect most likely means we read a detached element
    // already removed from the DOM (getBoundingClientRect is specified to return all
    // zeros in that case). In that situation, better to skip this slide transition
    // (the new tooltip follows the normal openDelay flow) than to treat (0,0) as a
    // valid slide-in origin and get the obviously wrong "slides in from the top-left
    // corner" look.
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
