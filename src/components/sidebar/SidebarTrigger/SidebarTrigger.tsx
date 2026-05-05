import { splitProps } from "solid-js";
import type { SidebarTriggerProps } from "./SidebarTrigger.types";
import { Button } from "~/components/button";
import { useSidebar } from "../SidebarProvider";
import { PanelLeft, PanelRight } from "lucide-solid";

export const SidebarTrigger = (props: SidebarTriggerProps) => {
  const { toggleSidebar, open } = useSidebar();
  const [local, others] = splitProps(props, ["class", "classList", "onClick"]);

  return (
    <Button
      data-sidebar="trigger"
      data-slot="sidebar-trigger"
      variant="ghost"
      size="sm"
      class={local.class}
      onClick={(event) => {
        local.onClick?.(event);
        toggleSidebar();
      }}
      icon={
        open() ? (
          <PanelLeft class="cn-rtl-flip" />
        ) : (
          <PanelRight class="cn-rtl-flip" />
        )
      }
      {...others}
    />
  );
};
