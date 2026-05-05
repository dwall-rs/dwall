import { splitProps } from "solid-js";
import type { ProgressValueProps } from "./ProgressValue.types";
import { clsx } from "~/utils";
import { classes } from "./ProgressValue.styles";
import { useProgressContext } from "../Progress";

export const ProgressValue = (props: ProgressValueProps) => {
  const { value } = useProgressContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <span
      data-slot="progress-value"
      class={clsx(classes, local.class)}
      {...others}
    >
      {value()}%
    </span>
  );
};
