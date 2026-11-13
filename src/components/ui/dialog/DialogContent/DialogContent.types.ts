import type { BaseProps, PolymorphicProps } from "~/types";

export type DialogContentProps = PolymorphicProps<
  "div",
  BaseProps & {
    showCloseButton?: boolean;

    dismissOnOverlayClick?: boolean;

    overlayClass?: string;
  },
  false
>;
