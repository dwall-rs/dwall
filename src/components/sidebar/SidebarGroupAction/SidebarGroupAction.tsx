import { mergeProps, splitProps, type ValidComponent } from "solid-js";
import type { SidebarGroupActionProps } from "./SidebarGroupAction.types";
import { clsx } from "~/utils";
import { Dynamic } from "solid-js/web";

export const SidebarGroupAction = <T extends ValidComponent = "button">(
  props: SidebarGroupActionProps<T>,
) => {
  const merged = mergeProps({ as: "button" as T } as const, props);

  const [local, others] = splitProps(merged, ["as", "class", "classList"]);

  return (
    // @ts-expect-error
    <Dynamic
      component={local.as}
      data-slot="sidebar-group-action"
      data-sidebar="group-action"
      class={clsx(
        "absolute top-3.5 right-3 flex aspect-square w-5 items-center justify-center rounded-md p-0 text-neutral-950 dark:text-neutral-50 ring-neutral-400 dark:ring-neutral-500 outline-hidden transition-transform group-data-[collapsible=icon]:hidden after:absolute after:-inset-2 hover:bg-neutral-100 dark:hover:bg-neutral-800 hover:text-neutral-900 dark:hover:text-neutral-50 focus-visible:ring-2 md:after:hidden [&>svg]:size-4 [&>svg]:shrink-0",
        local.class,
      )}
      {...others}
    />
  );
};
