import type { BaseProps, PolymorphicProps } from "~/types";

export type DialogOverlayProps = PolymorphicProps<
  "div",
  BaseProps & {
    /** 点击 Overlay 是否关闭，默认 true */
    dismissOnOverlayClick?: boolean;
  },
  false
>;
