import { splitProps } from "solid-js";
import type { SliderIndicatorProps } from "./SliderIndicator.types";
import { clsx } from "~/utils";
import { classes } from "./SliderIndicator.styles";
import { useSliderIndicator } from "./useSliderIndicator";
import { useSliderContext } from "../Slider";

export const SliderIndicator = (props: SliderIndicatorProps) => {
  const ctx = useSliderContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const { indicatorStyle } = useSliderIndicator();

  return (
    <div
      data-slot="slider-range"
      data-orientation={ctx.orientation()}
      class={clsx(classes, local.class)}
      style={indicatorStyle()}
      {...others}
    />
  );
};
