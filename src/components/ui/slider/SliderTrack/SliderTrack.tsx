import { splitProps } from "solid-js";
import type { SliderTrackProps } from "./SliderTrack.types";
import { clsx } from "~/utils";
import { classes } from "./SliderTrack.styles";
import { useSliderContext } from "../Slider";

export const SliderTrack = (props: SliderTrackProps) => {
  const ctx = useSliderContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  return (
    <div
      data-slot="slider-track"
      data-orientation={ctx.orientation()}
      ref={ctx.setTrackRef}
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
