import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseAspectRatioProps extends BaseProps {
  ratio: number;
}

export type AspectRatioProps = PolymorphicProps<
  "div",
  BaseAspectRatioProps,
  false
>;
