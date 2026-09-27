import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseSliderThumbProps {
  index: number;
}

export type SliderThumbProps = PolymorphicProps<
  "div",
  BaseProps & BaseSliderThumbProps,
  false
>;
