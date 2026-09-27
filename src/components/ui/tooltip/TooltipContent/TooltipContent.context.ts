import { createContext, useContext, type Accessor } from "solid-js";
import type { MiddlewareData, Placement } from "~/lib";

export interface TooltipContentContextValue {
  /** middlewareData produced by createPositioner; TooltipArrow reads the arrow middleware's computed offset from it */
  middlewareData: Accessor<MiddlewareData>;
  /** Accessor for the trigger element; TooltipArrow uses it to compute --arrow-offset */
  reference: Accessor<Element | undefined>;
  /** Final placement after flip adjustment, deciding which edge the arrow hugs and which way it points */
  placement: Accessor<Placement>;
  /** Used on TooltipArrow mount to register its DOM node with the arrow middleware (Element rather than HTMLElement, to support SVG arrows) */
  setArrowElement: (el: Element) => void;
  /**
   * State driving the enter/exit animations — same signal as the
   * data-state on TooltipContent's inner element. TooltipArrow should use it
   * to play its own animation instead of the implicit mechanism of "a child
   * of the inner element inheriting its transform/opacity animation" (the
   * inner zoom-in-95 animation adds a non-none transform, which per the CSS
   * spec makes it a new containing block for children — conflicting with the
   * arrow deliberately skipping it and using the outer element as its base,
   * making layer responsibilities unreliable while animating).
   */
  animationState: Accessor<"open" | "closed">;
}

export const TooltipContentContext =
  createContext<TooltipContentContextValue>();

export function useTooltipContentContext(
  component: string,
): TooltipContentContextValue {
  const ctx = useContext(TooltipContentContext);
  if (!ctx) {
    throw new Error(`<${component}> must be rendered inside <TooltipContent>`);
  }
  return ctx;
}
