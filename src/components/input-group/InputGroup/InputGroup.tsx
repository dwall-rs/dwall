import { splitProps } from "solid-js";
import type { InputGroupProps } from "./InputGroup.types";
import { clsx } from "~/utils";

export const InputGroup = (props: InputGroupProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);
  return (
    <div
      data-slot="input-group"
      role="group"
      class={clsx(
        "group/input-group relative flex h-8 w-full min-w-0 items-center rounded-lg border transition-colors outline-none in-data-[slot=combobox-content]:focus-within:border-inherit in-data-[slot=combobox-content]:focus-within:ring-0 has-disabled:opacity-50 has-[[data-slot=input-group-control]:focus-visible]:ring-3 has-[[data-slot][aria-invalid=true]]:ring-3 has-[>[data-align=block-end]]:h-auto has-[>[data-align=block-end]]:flex-col has-[>[data-align=block-start]]:h-auto has-[>[data-align=block-start]]:flex-col has-[>textarea]:h-auto has-[>[data-align=block-end]]:[&>input]:pt-3 has-[>[data-align=block-start]]:[&>input]:pb-3 has-[>[data-align=inline-end]]:[&>input]:pr-1.5 has-[>[data-align=inline-start]]:[&>input]:pl-1.5",
        "border-neutral-200 has-disabled:bg-neutral-200/50 has-[[data-slot=input-group-control]:focus-visible]:border-neutral-400 has-[[data-slot=input-group-control]:focus-visible]:ring-neutral-400/50 has-[[data-slot][aria-invalid=true]]:border-red-600 has-[[data-slot][aria-invalid=true]]:ring-red-600/20",
        "dark:border-white/15 dark:bg-white/4.5 dark:has-disabled:bg-white/12 dark:has-[[data-slot=input-group-control]:focus-visible]:border-neutral-500 dark:has-[[data-slot=input-group-control]:focus-visible]:ring-neutral-500/50 dark:has-[[data-slot][aria-invalid=true]]:border-red-400 dark:has-[[data-slot][aria-invalid=true]]:ring-red-400/40",
        local.class,
      )}
      {...others}
    />
  );
};
