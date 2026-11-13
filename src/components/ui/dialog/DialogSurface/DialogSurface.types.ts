import type { BaseProps, PolymorphicProps } from "~/types";

export type DialogSurfaceProps = PolymorphicProps<
  "div",
  BaseProps & {
    role?: "dialog" | "alertdialog";

    dismissOnOverlayClick?: boolean;

    overlayClass?: string;
  },
  false
>;
