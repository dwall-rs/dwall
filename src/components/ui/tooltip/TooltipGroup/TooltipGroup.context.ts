import { createContext, useContext } from "solid-js";
import type { Rect } from "~/lib";

export interface ActiveTooltipEntry {
  /** Reads this tooltip layer's current rect, used as the "slide-in start point" by the next tooltip that preempts it. */
  rect: () => Rect | undefined;
  /** Close immediately (skipping closeDelay and the exit animation) — called when switching to another trigger within the group. */
  forceClose: () => void;
}

export interface TooltipGroupContextValue {
  /** On open, register self as the group's currently active tooltip; returns an unregister function (called on close/unmount). */
  registerActive: (entry: ActiveTooltipEntry) => () => void;
  /**
   * Called when a tooltip within the group is about to open:
   * if the group already has an active tooltip, force-close it immediately, capture
   * its current rect and return it (used as the start point of the slide transition);
   * if the group has no active tooltip, return undefined and the caller should follow
   * the normal openDelay flow.
   *
   * The check uses "is there an active tooltip" rather than a time window like
   * "within N ms of the last close" to precisely match the scenario where "the old
   * one is still in its exit countdown while another trigger gets hovered" — in that
   * case the old tooltip hasn't technically closed yet (open() is still true), so a
   * time-window check would miss the coordination because it "hasn't closed and
   * hasn't notified"; querying "currently active" is more reliable.
   */
  claim: () => Rect | undefined;
}

export const TooltipGroupContext = createContext<TooltipGroupContextValue>();

/**
 * Note: unlike other useXxxContext hooks, this one does not throw when the context
 * is missing — <TooltipGroup> is optional, so a standalone <Tooltip> needs no wrapper.
 */
export function useTooltipGroupContext(): TooltipGroupContextValue | undefined {
  return useContext(TooltipGroupContext);
}
