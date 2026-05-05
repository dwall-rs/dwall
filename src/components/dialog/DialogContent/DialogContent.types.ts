import type { BaseProps, PolymorphicProps } from "~/types";

export type DialogContentProps = PolymorphicProps<
  "div",
  BaseProps & {
    showCloseButton?: boolean;
  },
  false
>;
