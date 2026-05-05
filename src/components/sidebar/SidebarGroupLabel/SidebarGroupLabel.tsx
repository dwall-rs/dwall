import { mergeProps, splitProps, type ValidComponent } from "solid-js";
import type { SidebarGroupLabelProps } from "./SidebarGroupLabel.types";
import { clsx } from "~/utils";
import { Dynamic } from "solid-js/web";

export const SidebarGroupLabel = <T extends ValidComponent = "div">(
  props: SidebarGroupLabelProps<T>,
) => {
  const merged = mergeProps({ as: "div" } as const, props);

  const [local, others] = splitProps(merged, ["as", "class", "classList"]);

  return (
    // @ts-expect-error
    <Dynamic
      component={local.as}
      data-slot="sidebar-group-label"
      data-sidebar="group-label"
      class={clsx(
        "flex h-8 shrink-0 items-center rounded-md px-2 text-xs font-medium text-neutral-950/70 dark:text-neutral-50/70 ring-neutral-400 dark:ring-neutral-500 outline-hidden transition-[margin,opacity] duration-200 ease-linear group-data-[collapsible=icon]:-mt-8 group-data-[collapsible=icon]:opacity-0 focus-visible:ring-2 [&>svg]:size-4 [&>svg]:shrink-0",
        local.class,
      )}
      {...others}
    />
  );
};
