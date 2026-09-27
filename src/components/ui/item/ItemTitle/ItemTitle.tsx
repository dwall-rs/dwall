import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemTitleProps } from "./ItemTitle.types";

export const ItemTitle = (props: ItemTitleProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="item-title"
      class={clsx(
        "line-clamp-1 flex w-fit items-center gap-2 text-sm leading-snug font-medium underline-offset-4",
        local.class,
      )}
      {...others}
    />
  );
};
