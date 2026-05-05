import { splitProps } from "solid-js";
import type { SidebarRailProps } from "./SidebarRail.types";
import { clsx } from "~/utils";
import { useSidebar } from "../SidebarProvider";

export const SidebarRail = (props: SidebarRailProps) => {
  const { toggleSidebar } = useSidebar();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <button
      data-sidebar="rail"
      data-slot="sidebar-rail"
      aria-label="Toggle Sidebar"
      tabIndex={-1}
      onClick={toggleSidebar}
      title="Toggle Sidebar"
      class={clsx(
        "absolute inset-y-0 z-20 hidden w-4 transition-all ease-linear group-data-[side=left]:-right-4 group-data-[side=right]:left-0 after:absolute after:inset-y-0 after:inset-s-1/2 after:w-0.5 hover:after:bg-neutral-200 dark:hover:after:bg-white/10 sm:flex ltr:-translate-x-1/2 rtl:-translate-x-1/2",
        "in-data-[side=left]:cursor-w-resize in-data-[side=right]:cursor-e-resize",
        "[[data-side=left][data-state=collapsed]_&]:cursor-e-resize [[data-side=right][data-state=collapsed]_&]:cursor-w-resize",
        "group-data-[collapsible=offcanvas]:translate-x-0 group-data-[collapsible=offcanvas]:after:left-full hover:group-data-[collapsible=offcanvas]:bg-neutral-50 dark:hover:group-data-[collapsible=offcanvas]:bg-neutral-900",
        "[[data-side=left][data-collapsible=offcanvas]_&]:-right-2",
        "[[data-side=right][data-collapsible=offcanvas]_&]:-left-2",
        local.class,
      )}
      {...others}
    />
  );
};
