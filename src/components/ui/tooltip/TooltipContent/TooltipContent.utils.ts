import {
  offset,
  flip,
  shift,
  arrow,
  hide,
  containingBlockOffset,
  getSide,
  getAlignment,
  type Middleware,
  type Placement,
  type Side,
  type Alignment,
} from "~/lib";

/**
 * Converts shadcn/Radix-style split side + align params into the core
 * library's `Placement` string (e.g. side='top' + align='start' →
 * 'top-start'). align 'center' (default, no offset) gets no suffix: it's side.
 */
export function toPhysicalSide(
  side: Side | "inline-start" | "inline-end",
  dir?: string,
): Side {
  if (side === "inline-start") {
    return dir === "rtl" ? "right" : "left";
  }
  if (side === "inline-end") {
    return dir === "rtl" ? "left" : "right";
  }
  return side;
}

export function toPlacement(
  side: Side | "inline-start" | "inline-end",
  align: Alignment | "center",
  dir?: string,
): Placement {
  const physical = toPhysicalSide(side, dir);
  return align === "center" ? physical : (`${physical}-${align}` as Placement);
}

/**
 * Computes which point the scale animation should anchor on (CSS
 * transform-origin) from the effective placement. The anchor should land on
 * "the edge closest to the trigger" — with placement 'top' (content above the
 * trigger) it is the content's bottom edge, so the scale looks like it "grows
 * out of the trigger" rather than scaling in place (the default origin, the
 * geometric center 50% 50%, has no visual link to the emergence direction).
 * Same on the cross axis: align start hugs left/top, end right/bottom.
 */
export function getTransformOrigin(placement: Placement): string {
  const side = getSide(placement);
  const align = getAlignment(placement);
  const crossAxisOrigin =
    align === "start" ? "0%" : align === "end" ? "100%" : "50%";

  switch (side) {
    case "top":
      return `${crossAxisOrigin} 100%`;
    case "bottom":
      return `${crossAxisOrigin} 0%`;
    case "left":
      return `100% ${crossAxisOrigin}`;
    case "right":
      return `0% ${crossAxisOrigin}`;
  }
}

export interface BuildTooltipMiddlewareOptions {
  sideOffset: number;
  /** Offset along the cross axis (the alignment direction), mapping to the offset middleware's crossAxis */
  alignOffset: number;
  collisionPadding: number;
  arrowElement: () => Element | undefined;
}

/**
 * Fixed recipe for the tooltip positioning pipeline:
 *
 * 1. offset      — sideOffset on the main axis, alignOffset on the cross
 * 2. flip        — flips to the opposite side when the main axis doesn't fit
 *    (no room above → below)
 * 3. shift       — if still overflowing after a flip, slides along the edge
 *    without changing sides
 * 4. arrow       — the arrow's offset relative to content, so it always points
 *    at the trigger's center
 * 5. hide        — marks hidden when the trigger is fully clipped by its own
 *    scroll container
 * 6. containingBlockOffset — corrects the position:fixed containing-block
 *    offset caused by ancestor transforms etc. (rare in demos; only in real
 *    projects with many transforms — see its own comments). It is last because
 *    it fixes the coordinates finally written into the DOM: earlier steps use
 *    one shared viewport space, unaffected by this offset.
 *
 * Deliberately no size middleware to cap max-height dynamically or scroll
 * overlong content — a tooltip should never show a scrollbar (not a general
 * limitation of this library but the nature of tooltips; shadcn/Radix behave
 * the same). Width falls back to `max-w-xs` in TooltipContent.tsx so text
 * wraps naturally; height is unrestricted, left to the content's height.
 *
 * There is also a technical reason: an earlier implementation used size to
 * compute max-height with overflow:auto for "scrollable overlong content",
 * but the inner element — once it had both overflow:auto and an enter
 * animation (zoom-in-95 adds a transform) — became a new containing block for
 * the arrow while animating (CSS spec: a non-none transform creates one), so
 * the arrow's overhang counted into its scrollable overflow: "a scrollbar
 * flashes on open and clips the arrow, then vanishes and the arrow returns
 * once the animation ends". Dropping scrolling severs this coupling at root.
 *
 * Order matters: arrow must come after shift/flip (it needs the final
 * coordinates), and containingBlockOffset after every other middleware.
 *
 * Extracted into its own function to keep useTooltipContent tidy and to test
 * this pipeline in isolation.
 */
export function createTooltipMiddleware(
  options: BuildTooltipMiddlewareOptions,
): Middleware[] {
  return [
    offset({ mainAxis: options.sideOffset, crossAxis: options.alignOffset }),
    flip(),
    shift({ padding: options.collisionPadding }),
    arrow({ element: options.arrowElement, padding: 6 }),
    hide(),
    containingBlockOffset(),
  ];
}
