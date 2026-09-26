import type { VariantProps } from "class-variance-authority";
import type { PolymorphicProps } from "~/types";
import type { inputGroupAddonVariants } from "./InputGroupAddon.styles";

export type InputGroupAddonProps = PolymorphicProps<
  "div",
  VariantProps<typeof inputGroupAddonVariants>,
  false
>;
