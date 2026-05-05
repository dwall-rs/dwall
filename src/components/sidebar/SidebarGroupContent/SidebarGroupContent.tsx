import { splitProps } from "solid-js";
import type { SidebarGroupContentProps } from "./SidebarGroupContent.types";
import { clsx } from "~/utils";

export const SidebarGroupContent = (props: SidebarGroupContentProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-group-content"
      data-sidebar="group-content"
      class={clsx("w-full text-sm", local.class)}
      {...others}
    />
  );
};
