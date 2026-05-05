import { splitProps } from "solid-js";
import type { SidebarContentProps } from "./SidebarContent.types";
import { clsx } from "~/utils";

export const SidebarContent = (props: SidebarContentProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-content"
      data-sidebar="content"
      class={clsx(
        "no-scrollbar flex min-h-0 flex-1 flex-col gap-0 overflow-auto group-data-[collapsible=icon]:overflow-hidden",
        local.class,
      )}
      {...others}
    />
  );
};
