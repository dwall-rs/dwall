import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { itemMediaVariants } from "./ItemMedia.styles";

export type ItemMediaProps = PolymorphicProps<
  "div",
  BaseProps & VariantProps<typeof itemMediaVariants>,
  false
>;
