import { splitProps } from "solid-js";
import type { SidebarMenuBadgeProps } from "./SidebarMenuBadge.types";
import { clsx } from "~/utils";

export const SidebarMenuBadge = (props: SidebarMenuBadgeProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-menu-badge"
      data-sidebar="menu-badge"
      class={clsx(
        "pointer-events-none absolute right-1 flex h-5 min-w-5 items-center justify-center rounded-md px-1 text-xs font-medium text-neutral-950 dark:text-neutral-50 tabular-nums select-none group-data-[collapsible=icon]:hidden peer-hover/menu-button:text-neutral-900 dark:peer-hover/menu-button:text-neutral-50 peer-data-[size=default]/menu-button:top-1.5 peer-data-[size=lg]/menu-button:top-2.5 peer-data-[size=sm]/menu-button:top-1 peer-data-active/menu-button:text-neutral-900 dark:peer-data-active/menu-button:text-neutral-50",
        local.class,
      )}
      {...others}
    />
  );
};
