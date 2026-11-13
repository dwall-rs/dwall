import type { BaseProps, PolymorphicProps } from "~/types";

export type DialogOverlayProps = PolymorphicProps<
  "div",
  BaseProps & {
    dismissOnOverlayClick?: boolean;
  },
  false
>;
