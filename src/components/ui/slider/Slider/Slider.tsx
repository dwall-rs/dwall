import { For, mergeProps, splitProps } from "solid-js";
import type { SliderProps } from "./Slider.types";
import { clsx } from "~/utils";
import { classes } from "./Slider.styles";
import { SliderControl } from "../SliderControl";
import { SliderTrack } from "../SliderTrack";
import { SliderIndicator } from "../SliderIndicator";
import { SliderThumb } from "../SliderThumb";
import { useSlider } from "./useSlider";
import { SliderContext } from "./Slider.context";

export const Slider = <T extends number | number[]>(props: SliderProps<T>) => {
  const merged = mergeProps(
    {
      orientation: "horizontal",
      max: 100,
      min: 0,
      step: 1,
      disabled: false,
    } as const,
    props,
  );

  const [local, others] = splitProps(merged, [
    "defaultValue",
    "value",
    "max",
    "min",
    "step",
    "orientation",
    "onValueChange",
    "disabled",
    "class",
    "classList",
  ]);

  const { context } = useSlider({
    get local() {
      return local;
    },
  });

  return (
    <SliderContext.Provider value={context}>
      <div
        data-slot="slider"
        data-orientation={local.orientation}
        class={clsx(classes, local.class)}
        {...others}
      >
        <SliderControl>
          <SliderTrack>
            <SliderIndicator />
          </SliderTrack>

          <For each={Array.from({ length: context.values().length })}>
            {(_, idx) => <SliderThumb index={idx()} />}
          </For>
        </SliderControl>
      </div>
    </SliderContext.Provider>
  );
};
