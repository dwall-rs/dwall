import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemDescriptionProps } from "./ItemDescription.types";

export const ItemDescription = (props: ItemDescriptionProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <p
      data-slot="item-description"
      class={clsx(
        "line-clamp-2 text-left text-sm leading-normal font-normal text-muted-foreground group-data-[size=xs]/item:text-xs [&>a]:underline [&>a]:underline-offset-4 [&>a:hover]:text-primary",
        local.class,
      )}
      {...others}
    />
  );
};
