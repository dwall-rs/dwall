import { cva } from "class-variance-authority";

/**
 * Styles for the ToggleGroup root container.
 *
 * The root element also emits `data-vertical` / `data-horizontal` (see
 * ToggleGroup.tsx), which activate selectors like `data-vertical:*` and
 * `group-data-vertical/toggle-group:*` — the item's "joined" state styles
 * depend on the latter two.
 */
export const toggleGroupVariants = cva(
  "group/toggle-group flex w-fit flex-row items-center gap-[--spacing(var(--gap))] rounded-lg data-[size=sm]:rounded-[min(var(--radius-md),10px)] data-vertical:flex-col data-vertical:items-stretch",
);
