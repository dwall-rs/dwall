import { splitProps } from "solid-js";
import type { SliderControlProps } from "./SliderControl.types";
import { clsx } from "~/utils";
import { classes } from "./SliderControl.styles";
import { useSliderControl } from "./useSliderControl";
import { useSliderContext } from "../Slider";

export const SliderControl = (props: SliderControlProps) => {
  const ctx = useSliderContext();

  const [local, others] = splitProps(props, ["class", "classList"]);

  const { handlePointerDown, handlePointerMove, handlePointerUp } =
    useSliderControl();

  return (
    <div
      data-orientation={ctx.orientation()}
      data-disabled={ctx.disabled()}
      onPointerDown={handlePointerDown}
      onPointerMove={handlePointerMove}
      onPointerUp={handlePointerUp}
      class={clsx(classes, local.class)}
      {...others}
    />
  );
};
