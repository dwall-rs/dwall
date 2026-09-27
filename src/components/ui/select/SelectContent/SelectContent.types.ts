import type { ParentProps } from "solid-js";
import type { Placement } from "~/lib";

export interface SelectContentProps extends ParentProps {
  /**
   * Preferred placement when there is no selected value yet (placeholder
   * state), defaults to 'bottom-start'. Once a value is selected the
   * positioning strategy switches to "selected-item alignment" (see
   * alignSelectedItem in SelectContent.utils.ts) and this prop has no effect
   * — which is also why SelectContent, unlike TooltipContent, was not changed
   * to separate side/align props: the "selected-item alignment" positioning
   * model has no notion of a side at all (the panel does not attach to one
   * side of the trigger; the selected item itself covers the trigger), so
   * forcing side/align onto it would be less accurate.
   */
  placement?: Placement;
  /** Minimum gap from the viewport edge (px), used by flip/shift collision detection, defaults to 8 */
  collisionPadding?: number;
  class?: string;
  /**
   * Any other native div attribute is passed through as-is to the rendered
   * element — the same convention as SelectTrigger.
   */
  [key: string]: any;
}
