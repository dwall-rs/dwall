import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemSeparatorProps } from "./ItemSeparator.types";
import { Separator } from "~/components/ui/separator";

export const ItemSeparator = (props: ItemSeparatorProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <Separator
      data-slot="item-separator"
      orientation="horizontal"
      class={clsx("my-2", local.class)}
      {...others}
    />
  );
};
