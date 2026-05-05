import type { VariantProps } from "class-variance-authority";
import type { PolymorphicProps } from "~/types";
import type { inputGroupButtonVariants } from "./InputGroupButton.styles";
import type { Button } from "~/components/button/Button";

export type InputGroupButtonProps = PolymorphicProps<
  typeof Button,
  VariantProps<typeof inputGroupButtonVariants> & {
    disabled?: boolean;
  },
  false
>;
