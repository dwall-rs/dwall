import type { BaseProps, PolymorphicProps } from "~/types";

export type ProgressProps = PolymorphicProps<
  "div",
  BaseProps & { value: number },
  false
>;
