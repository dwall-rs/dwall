import { splitProps } from "solid-js";
import type { SidebarInsetProps } from "./SidebarInset.types";
import { clsx } from "~/utils";

export const SidebarInset = (props: SidebarInsetProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <main
      data-slot="sidebar-inset"
      class={clsx(
        "relative flex w-full flex-1 flex-col bg-white dark:bg-neutral-950 md:peer-data-[variant=inset]:m-2 md:peer-data-[variant=inset]:ml-0 md:peer-data-[variant=inset]:rounded-xl md:peer-data-[variant=inset]:shadow-sm md:peer-data-[variant=inset]:peer-data-[state=collapsed]:ml-2",
        local.class,
      )}
      {...others}
    />
  );
};
