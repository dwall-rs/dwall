import { splitProps } from "solid-js";
import type { SidebarInputProps } from "./SidebarInput.types";
import { clsx } from "~/utils";
import { Input } from "~/components/input";

export const SidebarInput = (props: SidebarInputProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Input
      data-slot="sidebar-input"
      data-sidebar="input"
      class={clsx(
        "h-8 w-full bg-white dark:bg-neutral-950 shadow-none",
        local.class,
      )}
      {...others}
    />
  );
};
