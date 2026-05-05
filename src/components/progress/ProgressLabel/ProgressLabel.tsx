import { splitProps } from "solid-js";
import type { ProgressLabelProps } from "./ProgressLabel.types";
import { clsx } from "~/utils";
import { classes } from "./ProgressLabel.styles";

export const ProgressLabel = (props: ProgressLabelProps) => {
  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <span
      data-slot="progress-label"
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
