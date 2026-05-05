import { splitProps } from "solid-js";
import { clsx } from "~/utils";
import { classes } from "./AspectRatio.styles";
import type { AspectRatioProps } from "./AspectRatio.types";

export const AspectRatio = (props: AspectRatioProps) => {
  const [local, others] = splitProps(props, ["ratio", "class", "classList"]);

  return (
    <div
      data-slot="aspect-ratio"
      style={{ "--ratio": props.ratio }}
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
