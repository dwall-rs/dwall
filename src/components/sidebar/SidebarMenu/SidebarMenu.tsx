import { splitProps } from "solid-js";
import type { SidebarMenuProps } from "./SidebarMenu.types";
import { clsx } from "~/utils";

export const SidebarMenu = (props: SidebarMenuProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <ul
      data-slot="sidebar-menu"
      data-sidebar="menu"
      class={clsx("flex w-full min-w-0 flex-col gap-0", local.class)}
      {...others}
    />
  );
};
