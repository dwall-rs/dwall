import { mergeProps, splitProps, type ValidComponent } from "solid-js";
import { Dynamic } from "solid-js/web";
import type { SidebarMenuSubButtonProps } from "./SidebarMenuSubButton.types";
import { clsx } from "~/utils";

export const SidebarMenuSubButton = <T extends ValidComponent = "a">(
  props: SidebarMenuSubButtonProps<T>,
) => {
  const merged = mergeProps(
    { isActive: false, size: "md", as: "a" as T },
    props,
  );

  const [local, others] = splitProps(merged, [
    "as",
    "isActive",
    "size",
    "class",
    "classList",
  ]);

  return (
    // @ts-expect-error
    <Dynamic
      component={local.as}
      data-slot="sidebar-menu-sub-button"
      data-sidebar="menu-sub-button"
      data-size={local.size}
      data-active={local.isActive}
      class={clsx(
        "flex h-7 min-w-0 -translate-x-px items-center gap-2 overflow-hidden rounded-md px-2 text-neutral-950 dark:text-neutral-50 ring-neutral-400 dark:ring-neutral-500 outline-hidden group-data-[collapsible=icon]:hidden hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-900 dark:hover:text-neutral-50 focus-visible:ring-2 active:bg-neutral-100 dark:active:bg-neutral-800 active:text-neutral-900 dark:active:text-neutral-50 disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 data-[size=md]:text-sm data-[size=sm]:text-xs data-active:bg-neutral-100 dark:data-active:bg-neutral-800 data-active:text-neutral-900 dark:data-active:text-neutral-50 [&>span:last-child]:truncate [&>svg]:size-4 [&>svg]:shrink-0 [&>svg]:text-neutral-900 dark:[&>svg]:text-neutral-50",
        local.class,
      )}
      {...others}
    />
  );
};
