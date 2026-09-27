import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import type { ItemActionsProps } from "./ItemActions.types";

export const ItemActions = (props: ItemActionsProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <p
      data-slot="item-actions"
      class={clsx("flex items-center gap-2", local.class)}
      {...others}
    />
  );
};
