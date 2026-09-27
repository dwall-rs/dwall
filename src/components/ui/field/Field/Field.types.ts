import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { fieldVariants } from "./Field.styles";

export type FieldProps = PolymorphicProps<
  "div",
  BaseProps & VariantProps<typeof fieldVariants>,
  false
>;
