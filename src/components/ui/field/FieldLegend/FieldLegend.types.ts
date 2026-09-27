import type { BaseProps, PolymorphicProps } from "~/types";

export type FieldLegendProps = PolymorphicProps<
  "legend",
  BaseProps & { variant?: "legend" | "label" },
  false
>;
