import { splitProps } from "solid-js";
import type { SidebarMenuSubItemProps } from "./SidebarMenuSubItem.types";
import { clsx } from "~/utils";

export const SidebarMenuSubItem = (props: SidebarMenuSubItemProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <li
      data-slot="sidebar-menu-sub-item"
      data-sidebar="menu-sub-item"
      class={clsx("group/menu-sub-item relative", local.class)}
      {...others}
    />
  );
};
