import { splitProps } from "solid-js";
import type { SidebarMenuItemProps } from "./SidebarMenuItem.types";
import { clsx } from "~/utils";

export const SidebarMenuItem = (props: SidebarMenuItemProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <li
      data-slot="sidebar-menu-item"
      data-sidebar="menu-item"
      class={clsx("group/menu-item relative", local.class)}
      {...others}
    />
  );
};
