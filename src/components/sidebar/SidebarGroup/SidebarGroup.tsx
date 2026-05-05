import { splitProps } from "solid-js";
import type { SidebarGroupProps } from "./SidebarGroup.types";
import { clsx } from "~/utils";

export const SidebarGroup = (props: SidebarGroupProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-group"
      data-sidebar="group"
      class={clsx("relative flex w-full min-w-0 flex-col p-2", local.class)}
      {...others}
    />
  );
};
