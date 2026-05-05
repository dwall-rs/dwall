import { splitProps } from "solid-js";
import type { SidebarHeaderProps } from "./SidebarHeader.types";
import { clsx } from "~/utils";

export const SidebarHeader = (props: SidebarHeaderProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-header"
      data-sidebar="header"
      class={clsx("flex flex-col gap-2 p-2", local.class)}
      {...others}
    />
  );
};
