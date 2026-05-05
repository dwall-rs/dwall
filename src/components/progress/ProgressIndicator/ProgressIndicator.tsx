import { splitProps } from "solid-js";
import type { ProgressIndicatorProps } from "./ProgressIndicator.types";
import { clsx } from "~/utils";
import { classes } from "./ProgressIndicator.styles";
import { useProgressContext } from "../Progress";

export const ProgressIndicator = (props: ProgressIndicatorProps) => {
  const { value } = useProgressContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="progress-indicator"
      class={clsx(classes, "inset-s-0 h-[inherit]", local.class)}
      data-progressing={value() > 0 && value() < 100}
      style={{ width: `${value()}%` }}
      {...others}
    />
  );
};
