import { mergeProps, splitProps, type ValidComponent } from "solid-js";
import { clsx } from "~/utils";
import type { ItemProps } from "./Item.types";
import { itemVariants } from "./Item.styles";
import { Dynamic } from "solid-js/web";

export const Item = <T extends ValidComponent = "div">(props: ItemProps<T>) => {
  const merged = mergeProps(
    { variant: "ghost", size: "sm", component: "div" } as const,
    props,
  );

  const [local, others] = splitProps(merged, [
    "component",
    "variant",
    "size",
    "class",
    "classList",
  ]);

  return (
    <Dynamic
      component={local.component as ValidComponent}
      data-slot="item"
      class={clsx(
        itemVariants({ variant: local.variant, size: local.size }),
        local.class,
      )}
      {...others}
    />
  );
};
