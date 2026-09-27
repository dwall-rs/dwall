import type { VariantProps } from "class-variance-authority";
import type { BaseProps, PolymorphicProps } from "~/types";
import type { buttonGroupVariants } from "./ButtonGroup.styles";

type BaseButtonGroupProps = VariantProps<typeof buttonGroupVariants> &
  BaseProps;

export type ButtonGroupProps = PolymorphicProps<
  "fieldset",
  BaseButtonGroupProps,
  false
>;
