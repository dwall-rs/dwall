import type { BaseProps, PolymorphicProps } from "~/types";

export type AlertDialogContentProps = PolymorphicProps<
  "div",
  BaseProps & {
    size?: "md" | "sm";
  },
  false
>;
