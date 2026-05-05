import {
  children,
  mergeProps,
  Show,
  splitProps,
  type ValidComponent,
} from "solid-js";
import type { SidebarMenuButtonProps } from "./SidebarMenuButton.types";
import { clsx } from "~/utils";
import { useSidebar } from "../SidebarProvider";
import { Dynamic } from "solid-js/web";
import { sidebarMenuButtonVariants } from "./SidebarMenuButton.styles";
import { Tooltip, TooltipContent, TooltipTrigger } from "~/components/tooltip";

export const SidebarMenuButton = <T extends ValidComponent = "button">(
  props: SidebarMenuButtonProps<T>,
) => {
  const { isMobile, state } = useSidebar();

  const merged = mergeProps(
    { isActive: false, variant: "ghost", size: "md", as: "button" as T },
    props,
  );

  const [local, others] = splitProps(merged, [
    "as",
    "disabled",
    "isActive",
    "variant",
    "size",
    "tooltip",
    "class",
    "classList",
  ]);

  const comp = children(() => (
    // @ts-expect-error
    <Dynamic
      component={local.as}
      data-slot="sidebar-menu-button"
      data-sidebar="menu-button"
      data-size={local.size}
      data-active={local.isActive}
      data-disabled={local.disabled}
      aria-disabled={local.disabled}
      class={clsx(
        sidebarMenuButtonVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      {...others}
    />
  ));

  return (
    <Show when={local.tooltip} fallback={comp()}>
      <Tooltip>
        <TooltipTrigger>{comp()}</TooltipTrigger>
        <TooltipContent
          side="right"
          align="center"
          hidden={state() !== "collapsed" || isMobile}
          {...(typeof local.tooltip === "string"
            ? { children: local.tooltip }
            : local.tooltip)}
        />
      </Tooltip>
    </Show>
  );
};
