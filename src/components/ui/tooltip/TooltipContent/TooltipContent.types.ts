import type { Alignment, Side } from "~/lib";
import type { BaseProps, PolymorphicProps } from "~/types";

export type TooltipSide = Side | "inline-start" | "inline-end";

interface BaseTooltipContentProps extends BaseProps {
  /** Which side of the trigger it hugs, defaults to 'top'. Supports Base UI logical sides. */
  side?: TooltipSide;
  /** Alignment along that side, defaults to 'center' (no offset, centered) */
  align?: Alignment | "center";
  /** Gap in pixels from the trigger along the main axis (the direction side points), defaults to 8 */
  sideOffset?: number;
  /**
   * Offset in pixels along the cross axis (the alignment direction), defaults
   * to 0 — for fine-tuning the position when align is 'start'/'end'. Maps
   * directly to the offset middleware's crossAxis.
   */
  alignOffset?: number;
  /** Minimum gap in pixels from the viewport edge, used by flip/shift collision detection, defaults to 8 */
  collisionPadding?: number;
}

export type TooltipContentProps = PolymorphicProps<
  "div",
  BaseTooltipContentProps,
  false
>;
