import type { BaseProps, PolymorphicProps } from "~/types";

interface BaseSpinnerProps extends BaseProps {}

export type SpinnerProps = PolymorphicProps<"svg", BaseSpinnerProps, false>;
