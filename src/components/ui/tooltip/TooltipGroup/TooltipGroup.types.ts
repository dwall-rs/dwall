import type { ParentProps } from "solid-js";

/**
 * <TooltipGroup> is optional: wrapping multiple <Tooltip>s together makes switching
 * hover between adjacent triggers in the group skip openDelay and trigger a "slide
 * from the old position to the new one" transition, instead of the old tooltip
 * finishing its exit animation and the new one waiting for openDelay.
 *
 * If you don't need that effect, <Tooltip> works perfectly well on its own outside
 * <TooltipGroup> with unchanged behavior (internally it uses
 * useTooltipGroupContext to detect presence and degrades silently when absent).
 */
export type TooltipGroupProps = ParentProps;
