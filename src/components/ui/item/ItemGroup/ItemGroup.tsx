import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemGroupProps } from "./ItemGroup.types";

export const ItemGroup = (props: ItemGroupProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      role="list"
      data-slot="item-group"
      class={clsx(
        "group/item-group flex w-full flex-col gap-4 has-data-[size=sm]:gap-2.5 has-data-[size=xs]:gap-2",
        local.class,
      )}
      {...others}
    />
  );
};
