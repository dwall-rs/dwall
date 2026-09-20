import type { InputGroupAddonProps } from "./InputGroupAddon.types";
import { inputGroupAddonVariants } from "./InputGroupAddon.styles";
import { mergeProps, splitProps } from "solid-js";
import { clsx } from "~/utils";

export const InputGroupAddon = (props: InputGroupAddonProps) => {
  const merged = mergeProps({ align: "inline-start" } as const, props);

  const [local, others] = splitProps(merged, ["align", "class", "classList"]);

  return (
    <div
      role="group"
      data-slot="input-group-addon"
      data-align={local.align}
      class={clsx(inputGroupAddonVariants({ align: local.align }), local.class)}
      onClick={(e) => {
        if ((e.target as HTMLElement).closest("button")) {
          return;
        }
        e.currentTarget.parentElement?.querySelector("input")?.focus();
      }}
      {...others}
    />
  );
};
