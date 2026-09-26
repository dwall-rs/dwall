import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemFooterProps } from "./ItemFooter.types";

export const ItemFooter = (props: ItemFooterProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="item-footer"
      class={clsx(
        "flex basis-full items-center justify-between gap-2",
        local.class,
      )}
      {...others}
    />
  );
};
