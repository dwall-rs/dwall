import { splitProps } from "solid-js";
import type { SidebarMenuSubProps } from "./SidebarMenuSub.types";
import { clsx } from "~/utils";

export const SidebarMenuSub = (props: SidebarMenuSubProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <ul
      data-slot="sidebar-menu-sub"
      data-sidebar="menu-sub"
      class={clsx(
        "mx-3.5 flex min-w-0 translate-x-px flex-col gap-1 border-l border-neutral-200 dark:border-white/10 px-2.5 py-0.5 group-data-[collapsible=icon]:hidden",
        local.class,
      )}
      {...others}
    />
  );
};
