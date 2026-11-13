import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { itemVariants } from "./Item.styles";
import type { ValidComponent } from "solid-js";

export type ItemProps<T extends ValidComponent> = PolymorphicProps<
  T,
  BaseProps & VariantProps<typeof itemVariants>
>;
