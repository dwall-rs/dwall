import type { Accessor, ParentProps } from "solid-js";
import type { Rect } from "~/lib";

export interface TooltipProps extends ParentProps {
  /** Controlled open state; if omitted it is managed internally (uncontrolled mode) */
  open?: boolean;
  /** Initial state in uncontrolled mode, defaults to false */
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean) => void;
  /**
   * How long after mouse hover before opening (ms). Keyboard focus opening
   * ignores this delay — keyboard users should not be forced to wait, so focus
   * goes through openImmediate. Defaults to 700, near the system tooltip norm.
   */
  openDelay?: number;
  /**
   * How long after the mouse leaves before closing (ms), leaving time to move
   * onto the content (e.g. selectable/copyable text). Defaults to 150.
   */
  closeDelay?: number;
  /** Disable the whole tooltip (the trigger can also be disabled separately) */
  disabled?: boolean;
}

export interface TooltipContextValue {
  open: Accessor<boolean>;
  disabled: Accessor<boolean>;
  /** Links the trigger's aria-describedby with the content's id */
  contentId: string;
  reference: Accessor<Element | undefined>;
  setReference: (el: Element) => void;
  floating: Accessor<HTMLElement | undefined>;
  setFloating: (el: HTMLElement | undefined) => void;
  /** Delayed open per openDelay; if already open it does nothing but clear any pending close timer */
  requestOpen: () => void;
  /** Delayed close per closeDelay */
  requestClose: () => void;
  /** Skip the delay and open immediately (keyboard focus case) */
  openImmediate: () => void;
  /** Skip the delay and close immediately (Escape / blur case) */
  closeImmediate: () => void;
  /** Cancel any pending close timer without changing the current state (used when the pointer moves onto the content) */
  keepOpen: () => void;
  /**
   * On a <TooltipGroup> "preemption" hit, records the preempted old tooltip's
   * rect — TooltipContent uses it to compute an initial offset for the "slide
   * from the old position to the new position" transition. After consuming it
   * once, call clearPendingSlideFrom so a later autoUpdate reposition (scroll,
   * window resize, …) does not mis-trigger the slide-in animation again.
   */
  pendingSlideFrom: Accessor<Rect | undefined>;
  clearPendingSlideFrom: () => void;
  /**
   * Set to true when force-closed by <TooltipGroup> preemption —
   * TooltipContent's Presence logic uses it to decide whether to skip the exit
   * animation and unmount immediately (avoiding coexisting with the new
   * sliding-in tooltip). consumeSuppressExitAnimation resets it to false on
   * read, consuming it once, so later normal closes keep their exit animation.
   */
  suppressExitAnimation: Accessor<boolean>;
  consumeSuppressExitAnimation: () => boolean;
}
