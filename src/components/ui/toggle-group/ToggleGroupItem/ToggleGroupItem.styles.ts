import { cva } from "class-variance-authority";

/**
 * The "joined state" styles ToggleGroupItem needs when inside a ToggleGroup.
 *
 * Relies on hooks on the root element:
 * - `group-data-[spacing=0]/toggle-group`: with spacing 0, adjacent items join into one block
 * - `data-[spacing=0]`: the item's own data-spacing (shadcn also emits it on the item)
 * - `group-data-horizontal|vertical/toggle-group`: decides first/last rounded corners/borders by orientation
 * - `data-[variant=outline]`: for the outline variant when joined, keep only the inner border
 *
 * The base variant / size styles still come from `toggleVariants`; this only
 * fills in how the items combine.
 */
export const toggleGroupItemVariants = cva(
  "shrink-0 focus:z-10 focus-visible:z-10 group-data-[spacing=0]/toggle-group:rounded-none group-data-[spacing=0]/toggle-group:px-2 group-data-[spacing=0]/toggle-group:has-data-[icon=inline-end]:pr-1.5 group-data-[spacing=0]/toggle-group:has-data-[icon=inline-start]:pl-1.5 group-data-horizontal/toggle-group:data-[spacing=0]:first:rounded-l-lg group-data-vertical/toggle-group:data-[spacing=0]:first:rounded-t-lg group-data-horizontal/toggle-group:data-[spacing=0]:last:rounded-r-lg group-data-vertical/toggle-group:data-[spacing=0]:last:rounded-b-lg group-data-horizontal/toggle-group:data-[spacing=0]:data-[variant=outline]:border-l-0 group-data-vertical/toggle-group:data-[spacing=0]:data-[variant=outline]:border-t-0 group-data-horizontal/toggle-group:data-[spacing=0]:data-[variant=outline]:first:border-l group-data-vertical/toggle-group:data-[spacing=0]:data-[variant=outline]:first:border-t",
);
