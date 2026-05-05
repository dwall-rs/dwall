import { splitProps } from "solid-js";
import type { SidebarFooterProps } from "./SidebarFooter.types";
import { clsx } from "~/utils";

export const SidebarFooter = (props: SidebarFooterProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="sidebar-footer"
      data-sidebar="footer"
      class={clsx("flex flex-col gap-2 p-2", local.class)}
      {...others}
    />
  );
};
