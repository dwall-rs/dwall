import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemHeaderProps } from "./ItemHeader.types";

export const ItemHeader = (props: ItemHeaderProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="item-header"
      class={clsx(
        "flex basis-full items-center justify-between gap-2",
        local.class,
      )}
      {...others}
    />
  );
};
