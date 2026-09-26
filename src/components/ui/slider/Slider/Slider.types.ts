import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseSliderProps<T> {
  defaultValue?: T;
  value?: T;
  min?: number;
  max?: number;
  step?: number;
  orientation?: "horizontal" | "vertical";
  onValueChange?: (value: T) => void;
  disabled?: boolean;
}

export type SliderProps<T extends number | number[]> = PolymorphicProps<
  "div",
  BaseProps & BaseSliderProps<T>,
  false
>;
