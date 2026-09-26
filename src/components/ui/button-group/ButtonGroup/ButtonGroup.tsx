import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import { buttonGroupVariants } from "./ButtonGroup.styles";
import type { ButtonGroupProps } from "./ButtonGroup.types";

export const ButtonGroup = (props: ButtonGroupProps) => {
  const [local, others] = splitProps(props, [
    "orientation",
    "class",
    "classList",
  ]);
  return (
    <fieldset
      data-slot="button-group"
      data-orientation={local.orientation}
      class={clsx(
        buttonGroupVariants({ orientation: local.orientation }),
        local.class,
      )}
      {...others}
    />
  );
};
