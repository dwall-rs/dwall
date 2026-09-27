import { type Accessor, createContext, useContext } from "solid-js";

interface SliderContextValue {
  values: Accessor<number[]>;
  min: Accessor<number>;
  max: Accessor<number>;
  step: Accessor<number>;
  orientation: Accessor<"horizontal" | "vertical">;
  disabled: Accessor<boolean>;
  activeThumbIndex: Accessor<number>;
  setActiveThumbIndex: (index: number) => void;
  updateValue: (index: number, value: number) => void;
  trackRef: Accessor<HTMLDivElement | undefined>;
  setTrackRef: (el: HTMLDivElement) => void;
}

export const SliderContext = createContext<SliderContextValue>();

export const useSliderContext = () => {
  const ctx = useContext(SliderContext);
  if (!ctx) {
    throw new Error("useSliderContext must be used within a SliderProvider");
  }

  return ctx;
};
