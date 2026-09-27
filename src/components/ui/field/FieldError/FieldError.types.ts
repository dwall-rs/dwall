import type { BaseProps, PolymorphicProps } from "~/types";

export type FieldErrorProps = PolymorphicProps<
  "div",
  BaseProps & {
    errors?: Array<{ message?: string } | undefined>;
  },
  false
>;
