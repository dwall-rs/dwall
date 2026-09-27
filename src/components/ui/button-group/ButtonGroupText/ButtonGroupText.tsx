import { clsx } from "~/utils";
import classes from "./ButtonGroupText.styles";
import type { ButtonGroupTextProps } from "./ButtonGroupText.types";

export const ButtonGroupText = (props: ButtonGroupTextProps) => {
  return (
    <div
      data-slot="button-group-text"
      class={clsx(classes, props.class)}
      {...props}
    />
  );
};
