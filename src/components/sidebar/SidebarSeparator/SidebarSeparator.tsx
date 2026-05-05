import { splitProps } from "solid-js";
import type { SidebarSeparatorProps } from "./SidebarSeparator.types";
import { clsx } from "~/utils";
import { Separator } from "~/components/separator";

export const SidebarSeparator = (props: SidebarSeparatorProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Separator
      data-slot="sidebar-separator"
      data-sidebar="separator"
      class={clsx("mx-2 w-auto bg-sidebar-border", local.class)}
      {...others}
    />
  );
};
