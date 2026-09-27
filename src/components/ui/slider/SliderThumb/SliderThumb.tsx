import { splitProps } from "solid-js";
import type { SliderThumbProps } from "./SliderThumb.types";
import { clsx } from "~/utils";
import { classes } from "./SliderThumb.styles";
import { useSliderThumb } from "./useSliderThumb";
import { useSliderContext } from "../Slider";

export const SliderThumb = (props: SliderThumbProps) => {
  const ctx = useSliderContext();

  const [local, others] = splitProps(props, ["index", "class", "classList"]);

  const { positionStyle, handleKeyDown } = useSliderThumb(local.index);

  return (
    <div
      data-slot="slider-thumb"
      data-index={local.index}
      data-orientation={ctx.orientation()}
      data-disabled={ctx.disabled()}
      aria-disabled={ctx.disabled()}
      tabindex={ctx.disabled() ? -1 : 0}
      class={clsx(classes, local.class)}
      style={positionStyle()}
      onKeyDown={handleKeyDown}
      onFocus={() => ctx.setActiveThumbIndex(local.index)}
      {...others}
    />
  );
};
