import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemContentProps } from "./ItemContent.types";

export const ItemContent = (props: ItemContentProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="item-content"
      class={clsx(
        "flex flex-1 flex-col gap-1 group-data-[size=xs]/item:gap-0 [&+[data-slot=item-content]]:flex-none",
        local.class,
      )}
      {...others}
    />
  );
};
