import { mergeProps, splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemMediaProps } from "./ItemMedia.types";
import { itemMediaVariants } from "./ItemMedia.styles";

export const ItemMedia = (props: ItemMediaProps) => {
  const merged = mergeProps({ variant: "ghost" } as const, props);

  const [local, others] = splitProps(merged, ["variant", "class", "classList"]);

  return (
    <div
      data-slot="item-media"
      data-variant={local.variant}
      class={clsx(itemMediaVariants({ variant: local.variant }), local.class)}
      {...others}
    />
  );
};
